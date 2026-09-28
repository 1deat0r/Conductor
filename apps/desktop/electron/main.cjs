const { app, BrowserWindow, protocol, net, session } = require('electron');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const fs = require('node:fs/promises');
protocol.registerSchemesAsPrivileged([{scheme:'conductor',privileges:{standard:true,secure:true,supportFetchAPI:true}}]);
app.enableSandbox();
const smoke = process.argv.includes('--smoke-test');
let window;
async function createWindow() {
  window = new BrowserWindow({width:1280,height:880,minWidth:820,minHeight:600,show:!smoke,backgroundColor:'#111815',webPreferences:{nodeIntegration:false,contextIsolation:true,sandbox:true,webSecurity:true}});
  window.webContents.setWindowOpenHandler(()=>({action:'deny'}));
  window.webContents.on('will-navigate',(event)=>event.preventDefault());
  if (smoke) {
    const timer=setTimeout(()=>{console.error('Desktop smoke timed out');app.exit(1);},20000);
    window.webContents.once('did-fail-load',()=>{clearTimeout(timer);app.exit(1);});
    window.webContents.once('did-finish-load',async()=>{
      try {
        const state=await window.webContents.executeJavaScript("({title:document.title,text:document.body.innerText,hasNode:typeof process!=='undefined'})");
        if(state.title!=='Conductor'||!state.text.includes('No tasks yet')||state.hasNode) throw new Error('Unexpected renderer state');
        console.log(JSON.stringify({desktop_smoke:'passed',sandbox:true,node_integration:false}));clearTimeout(timer);app.exit(0);
      } catch(error){console.error(error);clearTimeout(timer);app.exit(1);}
    });
  }
  await window.loadURL('conductor://app/index.html');
}
app.whenReady().then(async()=>{
  session.defaultSession.setPermissionRequestHandler((_webContents,_permission,callback)=>callback(false));
  session.defaultSession.setPermissionCheckHandler(()=>false);
  session.defaultSession.on('will-download',(event)=>event.preventDefault());
  const root=path.resolve(__dirname,'../dist');
  protocol.handle('conductor',async(request)=>{
    const url=new URL(request.url);
    if(url.host!=='app'||request.method!=='GET') return new Response('Forbidden',{status:403});
    let file;
    try{file=path.resolve(root,'.'+decodeURIComponent(url.pathname));}catch{return new Response('Bad path',{status:400});}
    if(!file.startsWith(root+path.sep)) return new Response('Forbidden',{status:403});
    try{const real=await fs.realpath(file);if(!real.startsWith(root+path.sep)) return new Response('Forbidden',{status:403});return net.fetch(pathToFileURL(real).toString());}catch{return new Response('Not found',{status:404});}
  });
  await createWindow();
  app.on('activate',()=>{if(BrowserWindow.getAllWindows().length===0)void createWindow();});
});
app.on('window-all-closed',()=>{if(process.platform!=='darwin')app.quit();});
