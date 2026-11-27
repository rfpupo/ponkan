//! Apply orchestration: backend boundaries can be exercised without a Riot session.
use std::future::Future;
type R<T> = Result<T, String>;

pub(crate) trait Backend: Sync {
    fn emergency_backup(&self, before: &str) -> R<()>;
    fn game_closed(&self) -> R<()>;
    fn assert_session(&self) -> impl Future<Output = R<()>> + Send;
    fn read_cloud(&self) -> impl Future<Output = R<String>> + Send;
    fn cloud_equals(&self, left: &str, right: &str) -> bool;
    fn verify_before(&self) -> R<()>;
    fn begin(&self, before: &str, intended: &str) -> R<()>;
    fn write_cloud(&self, data: &str) -> impl Future<Output = R<()>> + Send;
    fn rollback_cloud(&self, before: &str, intended: &str) -> impl Future<Output = R<()>> + Send;
    fn commit(&self) -> R<()>;
    fn verify_after(&self) -> R<()>;
    fn reconcile(&self) -> impl Future<Output = R<()>> + Send;
    fn finish(&self);
}

pub(crate) async fn run<B: Backend>(
    backend: &B,
    before: &str,
    intended: &str,
    auto_backup: bool,
) -> R<()> {
    if auto_backup {
        backend.emergency_backup(before)?;
    }
    backend.game_closed()?;
    backend.assert_session().await?;
    backend.verify_before()?;
    backend.begin(before, intended)?;
    let mut attempted = false;
    let result = async {
        backend.game_closed()?;
        backend.assert_session().await?;
        if !backend.cloud_equals(&backend.read_cloud().await?, before) {
            return Err("As configurações da nuvem mudaram durante a operação".into());
        }
        backend.verify_before()?;
        backend.game_closed()?;
        backend.assert_session().await?;
        if !backend.cloud_equals(before, intended) {
            attempted = true;
            backend.write_cloud(intended).await?;
        }
        backend.assert_session().await?;
        backend.game_closed()?;
        backend.commit()
    }
    .await;
    if let Err(error) = result {
        if attempted {
            // Never initiate recovery writes after the game reopens.
            if let Err(rollback) = backend.game_closed() {
                return Err(format!(
                    "{error}. Recuperação da nuvem bloqueada: {rollback}"
                ));
            }
            if let Err(rollback) = backend.rollback_cloud(before, intended).await {
                return Err(format!("{error}. Recuperação da nuvem falhou: {rollback}"));
            }
        }
        if let Err(recovery) = backend.reconcile().await {
            return Err(format!("{error}. {recovery}"));
        }
        return Err(error);
    }
    // Never clear the journal before both independently verified states agree.
    backend.assert_session().await?;
    backend.game_closed()?;
    if !backend.cloud_equals(&backend.read_cloud().await?, intended) {
        return Err("A nuvem não confirmou o perfil aplicado. Backup preservado.".into());
    }
    backend.verify_after()?;
    backend.reconcile().await?;
    backend.finish();
    Ok(())
}

