// Public source integrity checks. This does not authorize a release.
import fs from "node:fs";
import path from "node:path";
import {createHash} from "node:crypto";
import {fileURLToPath,pathToFileURL} from "node:url";
const root=fileURLToPath(new URL("..",import.meta.url));
const read=(base,rel)=>fs.readFileSync(path.join(base,rel),"utf8");
const json=(base,rel)=>JSON.parse(read(base,rel));
export function verifyPublicSource(base=root){
 const pkg=json(base,"package.json"),lock=json(base,"package-lock.json");
 const cargo=read(base,"src-tauri/Cargo.toml").split(/^\[/m)[1]??"";
 const cargoVersion=cargo.match(/^version\s*=\s*"([^"]+)"\s*$/m)?.[1];
 const cargoEntry=read(base,"src-tauri/Cargo.lock").split("[[package]]").find(s=>/^name = "ponkan"\s*$/m.test(s));
 const cargoLockVersion=cargoEntry?.match(/^version = "([^"]+)"/m)?.[1];
 const versions=[pkg.version,lock.version,lock.packages?.[""]?.version,cargoVersion,cargoLockVersion,json(base,"src-tauri/tauri.conf.json").version];
 if(versions.some(v=>typeof v!=="string"||v!==versions[0]||!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(v)))throw Error("Version metadata mismatch");
 const notices=json(base,"third-party-notices/INDEX.json");
 if(!Array.isArray(notices)||!notices.length)throw Error("Notice inventory missing");
 const names=new Map(), caveats=new Set();
 for(const item of notices){
  const key=item.ecosystem+":"+item.name+"@"+item.version;
  if(names.has(key)&&names.get(key)!==item.license)throw Error("Conflicting notice license: "+key);
  names.set(key,item.license);
  if(!item.license||item.license==="UNSPECIFIED"||!Array.isArray(item.notices)||!item.notices.length)throw Error("Missing notice: "+key);
  for(const n of item.notices){
   if(!/^[a-f0-9]{64}\.txt$/.test(n.notice))throw Error("Invalid notice path");
   const file=path.join(base,"third-party-notices",n.notice),stat=fs.lstatSync(file);
   if(!stat.isFile()||stat.isSymbolicLink()||createHash("sha256").update(fs.readFileSync(file)).digest("hex")!==n.notice.slice(0,-4))throw Error("Notice hash mismatch: "+key);
  }
  if(item.noticeProvenance?.includes("requires review"))caveats.add(key);
 }
 return {version:versions[0],noticeRecords:notices.length,uniquePackages:names.size,provenanceReview:[...caveats]};
}
if(process.argv[1]&&pathToFileURL(path.resolve(process.argv[1])).href===import.meta.url){
 try{console.log("PUBLIC_SOURCE_INTEGRITY",JSON.stringify(verifyPublicSource()));}
 catch(e){console.error(e.message);process.exitCode=1;}
}
