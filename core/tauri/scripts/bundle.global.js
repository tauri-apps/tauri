var __TAURI_IIFE__=function(e){"use strict";function t(e,t,n,i){if("a"===n&&!i)throw new TypeError("Private accessor was defined without a getter");if("function"==typeof t?e!==t||!i:!t.has(e))throw new TypeError("Cannot read private member from an object whose class did not declare it");return"m"===n?i:"a"===n?i.call(e):i?i.value:t.get(e)}function n(e,t,n,i,r){if("m"===i)throw new TypeError("Private method is not writable");if("a"===i&&!r)throw new TypeError("Private accessor was defined without a setter");if("function"==typeof t?e!==t||!r:!t.has(e))throw new TypeError("Cannot write private member to an object whose class did not declare it");return"a"===i?r.call(e,n):r?r.value=n:t.set(e,n),n}var i,r,a,s;function l(e,t=!1){return window.__TAURI_INTERNALS__.transformCallback(e,t)}"function"==typeof SuppressedError&&SuppressedError;class o{constructor(){this.__TAURI_CHANNEL_MARKER__=!0,i.set(this,(()=>{})),r.set(this,0),a.set(this,{}),this.id=l((({message:e,id:s})=>{if(s===t(this,r,"f")){n(this,r,s+1,"f"),t(this,i,"f").call(this,e);const l=Object.keys(t(this,a,"f"));if(l.length>0){let e=s+1;for(const n of l.sort()){if(parseInt(n)!==e)break;{const r=t(this,a,"f")[n];delete t(this,a,"f")[n],t(this,i,"f").call(this,r),e+=1}}n(this,r,e,"f")}}else t(this,a,"f")[s.toString()]=e}))}set onmessage(e){n(this,i,e,"f")}get onmessage(){return t(this,i,"f")}toJSON(){return`__CHANNEL__:${this.id}`}}i=new WeakMap,r=new WeakMap,a=new WeakMap;class u{constructor(e,t,n){this.plugin=e,this.event=t,this.channelId=n}async unregister(){return c(`plugin:${this.plugin}|remove_listener`,{event:this.event,channelId:this.channelId})}}async function c(e,t={},n){return window.__TAURI_INTERNALS__.invoke(e,t,n)}class d{get rid(){return t(this,s,"f")}constructor(e){s.set(this,void 0),n(this,s,e,"f")}async close(){return c("plugin:resources|close",{rid:this.rid})}}s=new WeakMap;var p=Object.freeze({__proto__:null,Channel:o,PluginListener:u,Resource:d,addPluginListener:async function(e,t,n){const i=new o;return i.onmessage=n,c(`plugin:${e}|register_listener`,{event:t,handler:i}).then((()=>new u(e,t,i.id)))},convertFileSrc:function(e,t="asset"){return window.__TAURI_INTERNALS__.convertFileSrc(e,t)},invoke:c,isTauri:function(){return"isTauri"in window&&!!window.isTauri},transformCallback:l});class h extends d{constructor(e){super(e)}static async new(e,t,n){return c("plugin:image|new",{rgba:w(e),width:t,height:n}).then((e=>new h(e)))}static async fromBytes(e){return c("plugin:image|from_bytes",{bytes:w(e)}).then((e=>new h(e)))}static async fromPath(e){return c("plugin:image|from_path",{path:e}).then((e=>new h(e)))}async rgba(){return c("plugin:image|rgba",{rid:this.rid}).then((e=>new Uint8Array(e)))}async size(){return c("plugin:image|size",{rid:this.rid})}}function w(e){return null==e?null:"string"==typeof e?e:e instanceof Uint8Array?Array.from(e):e instanceof ArrayBuffer?Array.from(new Uint8Array(e)):e instanceof h?e.rid:e}var y=Object.freeze({__proto__:null,Image:h,transformImage:w});var _=Object.freeze({__proto__:null,defaultWindowIcon:async function(){return c("plugin:app|default_window_icon").then((e=>e?new h(e):null))},getName:async function(){return c("plugin:app|name")},getTauriVersion:async function(){return c("plugin:app|tauri_version")},getVersion:async function(){return c("plugin:app|version")},hide:async function(){return c("plugin:app|app_hide")},show:async function(){return c("plugin:app|app_show")}});class g{constructor(e,t){this.type="Logical",this.width=e,this.height=t}}class b{constructor(e,t){this.type="Physical",this.width=e,this.height=t}toLogical(e){return new g(this.width/e,this.height/e)}}class m{constructor(e,t){this.type="Logical",this.x=e,this.y=t}}class v{constructor(e,t){this.type="Physical",this.x=e,this.y=t}toLogical(e){return new m(this.x/e,this.y/e)}}var f,k=Object.freeze({__proto__:null,LogicalPosition:m,LogicalSize:g,PhysicalPosition:v,PhysicalSize:b});async function A(e,t){await c("plugin:event|unlisten",{event:e,eventId:t})}async function E(e,t,n){var i;const r="string"==typeof(null==n?void 0:n.target)?{kind:"AnyLabel",label:n.target}:null!==(i=null==n?void 0:n.target)&&void 0!==i?i:{kind:"Any"};return c("plugin:event|listen",{event:e,target:r,handler:l(t)}).then((t=>async()=>A(e,t)))}async function D(e,t,n){return E(e,(n=>{A(e,n.id),t(n)}),n)}async function I(e,t){await c("plugin:event|emit",{event:e,payload:t})}async function T(e,t,n){const i="string"==typeof e?{kind:"AnyLabel",label:e}:e;await c("plugin:event|emit_to",{target:i,event:t,payload:n})}!function(e){e.WINDOW_RESIZED="tauri://resize",e.WINDOW_MOVED="tauri://move",e.WINDOW_CLOSE_REQUESTED="tauri://close-requested",e.WINDOW_DESTROYED="tauri://destroyed",e.WINDOW_FOCUS="tauri://focus",e.WINDOW_BLUR="tauri://blur",e.WINDOW_SCALE_FACTOR_CHANGED="tauri://scale-change",e.WINDOW_THEME_CHANGED="tauri://theme-changed",e.WINDOW_CREATED="tauri://window-created",e.WEBVIEW_CREATED="tauri://webview-created",e.DRAG="tauri://drag",e.DROP="tauri://drop",e.DROP_OVER="tauri://drop-over",e.DROP_CANCELLED="tauri://drag-cancelled"}(f||(f={}));var S,L,R,P,C,N=Object.freeze({__proto__:null,get TauriEvent(){return f},emit:I,emitTo:T,listen:E,once:D});function x(e){var t;if("items"in e)e.items=null===(t=e.items)||void 0===t?void 0:t.map((e=>"rid"in e?e:x(e)));else if("action"in e&&e.action){const t=new o;return t.onmessage=e.action,delete e.action,{...e,handler:t}}return e}async function z(e,t){const n=new o;let i=null;return t&&"object"==typeof t&&("action"in t&&t.action&&(n.onmessage=t.action,delete t.action),"items"in t&&t.items&&(i=t.items.map((e=>{var t;return"rid"in e?[e.rid,e.kind]:("item"in e&&"object"==typeof e.item&&(null===(t=e.item.About)||void 0===t?void 0:t.icon)&&(e.item.About.icon=w(e.item.About.icon)),"icon"in e&&e.icon&&(e.icon=w(e.icon)),x(e))})))),c("plugin:menu|new",{kind:e,options:t?{...t,items:i}:void 0,handler:n})}class W extends d{get id(){return t(this,S,"f")}get kind(){return t(this,L,"f")}constructor(e,t,i){super(e),S.set(this,void 0),L.set(this,void 0),n(this,S,t,"f"),n(this,L,i,"f")}}S=new WeakMap,L=new WeakMap;class O extends W{constructor(e,t){super(e,t,"MenuItem")}static async new(e){return z("MenuItem",e).then((([e,t])=>new O(e,t)))}async text(){return c("plugin:menu|text",{rid:this.rid,kind:this.kind})}async setText(e){return c("plugin:menu|set_text",{rid:this.rid,kind:this.kind,text:e})}async isEnabled(){return c("plugin:menu|is_enabled",{rid:this.rid,kind:this.kind})}async setEnabled(e){return c("plugin:menu|set_enabled",{rid:this.rid,kind:this.kind,enabled:e})}async setAccelerator(e){return c("plugin:menu|set_accelerator",{rid:this.rid,kind:this.kind,accelerator:e})}}class F extends W{constructor(e,t){super(e,t,"Check")}static async new(e){return z("Check",e).then((([e,t])=>new F(e,t)))}async text(){return c("plugin:menu|text",{rid:this.rid,kind:this.kind})}async setText(e){return c("plugin:menu|set_text",{rid:this.rid,kind:this.kind,text:e})}async isEnabled(){return c("plugin:menu|is_enabled",{rid:this.rid,kind:this.kind})}async setEnabled(e){return c("plugin:menu|set_enabled",{rid:this.rid,kind:this.kind,enabled:e})}async setAccelerator(e){return c("plugin:menu|set_accelerator",{rid:this.rid,kind:this.kind,accelerator:e})}async isChecked(){return c("plugin:menu|is_checked",{rid:this.rid})}async setChecked(e){return c("plugin:menu|set_checked",{rid:this.rid,checked:e})}}!function(e){e.Add="Add",e.Advanced="Advanced",e.Bluetooth="Bluetooth",e.Bookmarks="Bookmarks",e.Caution="Caution",e.ColorPanel="ColorPanel",e.ColumnView="ColumnView",e.Computer="Computer",e.EnterFullScreen="EnterFullScreen",e.Everyone="Everyone",e.ExitFullScreen="ExitFullScreen",e.FlowView="FlowView",e.Folder="Folder",e.FolderBurnable="FolderBurnable",e.FolderSmart="FolderSmart",e.FollowLinkFreestanding="FollowLinkFreestanding",e.FontPanel="FontPanel",e.GoLeft="GoLeft",e.GoRight="GoRight",e.Home="Home",e.IChatTheater="IChatTheater",e.IconView="IconView",e.Info="Info",e.InvalidDataFreestanding="InvalidDataFreestanding",e.LeftFacingTriangle="LeftFacingTriangle",e.ListView="ListView",e.LockLocked="LockLocked",e.LockUnlocked="LockUnlocked",e.MenuMixedState="MenuMixedState",e.MenuOnState="MenuOnState",e.MobileMe="MobileMe",e.MultipleDocuments="MultipleDocuments",e.Network="Network",e.Path="Path",e.PreferencesGeneral="PreferencesGeneral",e.QuickLook="QuickLook",e.RefreshFreestanding="RefreshFreestanding",e.Refresh="Refresh",e.Remove="Remove",e.RevealFreestanding="RevealFreestanding",e.RightFacingTriangle="RightFacingTriangle",e.Share="Share",e.Slideshow="Slideshow",e.SmartBadge="SmartBadge",e.StatusAvailable="StatusAvailable",e.StatusNone="StatusNone",e.StatusPartiallyAvailable="StatusPartiallyAvailable",e.StatusUnavailable="StatusUnavailable",e.StopProgressFreestanding="StopProgressFreestanding",e.StopProgress="StopProgress",e.TrashEmpty="TrashEmpty",e.TrashFull="TrashFull",e.User="User",e.UserAccounts="UserAccounts",e.UserGroup="UserGroup",e.UserGuest="UserGuest"}(R||(R={}));class U extends W{constructor(e,t){super(e,t,"Icon")}static async new(e){return z("Icon",e).then((([e,t])=>new U(e,t)))}async text(){return c("plugin:menu|text",{rid:this.rid,kind:this.kind})}async setText(e){return c("plugin:menu|set_text",{rid:this.rid,kind:this.kind,text:e})}async isEnabled(){return c("plugin:menu|is_enabled",{rid:this.rid,kind:this.kind})}async setEnabled(e){return c("plugin:menu|set_enabled",{rid:this.rid,kind:this.kind,enabled:e})}async setAccelerator(e){return c("plugin:menu|set_accelerator",{rid:this.rid,kind:this.kind,accelerator:e})}async setIcon(e){return c("plugin:menu|set_icon",{rid:this.rid,icon:w(e)})}}class M extends W{constructor(e,t){super(e,t,"Predefined")}static async new(e){return z("Predefined",e).then((([e,t])=>new M(e,t)))}async text(){return c("plugin:menu|text",{rid:this.rid,kind:this.kind})}async setText(e){return c("plugin:menu|set_text",{rid:this.rid,kind:this.kind,text:e})}}!function(e){e[e.Critical=1]="Critical",e[e.Informational=2]="Informational"}(P||(P={}));class B{constructor(e){this._preventDefault=!1,this.event=e.event,this.id=e.id}preventDefault(){this._preventDefault=!0}isPreventDefault(){return this._preventDefault}}function j(){return new H(window.__TAURI_INTERNALS__.metadata.currentWindow.label,{skip:!0})}function V(){return window.__TAURI_INTERNALS__.metadata.windows.map((e=>new H(e.label,{skip:!0})))}!function(e){e.None="none",e.Normal="normal",e.Indeterminate="indeterminate",e.Paused="paused",e.Error="error"}(C||(C={}));const G=["tauri://created","tauri://error"];class H{constructor(e,t={}){var n;this.label=e,this.listeners=Object.create(null),(null==t?void 0:t.skip)||c("plugin:window|create",{options:{...t,parent:"string"==typeof t.parent?t.parent:null===(n=t.parent)||void 0===n?void 0:n.label,label:e}}).then((async()=>this.emit("tauri://created"))).catch((async e=>this.emit("tauri://error",e)))}static getByLabel(e){var t;return null!==(t=V().find((t=>t.label===e)))&&void 0!==t?t:null}static getCurrent(){return j()}static getAll(){return V()}static async getFocusedWindow(){for(const e of V())if(await e.isFocused())return e;return null}async listen(e,t){return this._handleTauriEvent(e,t)?()=>{const n=this.listeners[e];n.splice(n.indexOf(t),1)}:E(e,t,{target:{kind:"Window",label:this.label}})}async once(e,t){return this._handleTauriEvent(e,t)?()=>{const n=this.listeners[e];n.splice(n.indexOf(t),1)}:D(e,t,{target:{kind:"Window",label:this.label}})}async emit(e,t){if(!G.includes(e))return I(e,t);for(const n of this.listeners[e]||[])n({event:e,id:-1,payload:t})}async emitTo(e,t,n){if(!G.includes(t))return T(e,t,n);for(const e of this.listeners[t]||[])e({event:t,id:-1,payload:n})}_handleTauriEvent(e,t){return!!G.includes(e)&&(e in this.listeners?this.listeners[e].push(t):this.listeners[e]=[t],!0)}async scaleFactor(){return c("plugin:window|scale_factor",{label:this.label})}async innerPosition(){return c("plugin:window|inner_position",{label:this.label}).then((({x:e,y:t})=>new v(e,t)))}async outerPosition(){return c("plugin:window|outer_position",{label:this.label}).then((({x:e,y:t})=>new v(e,t)))}async innerSize(){return c("plugin:window|inner_size",{label:this.label}).then((({width:e,height:t})=>new b(e,t)))}async outerSize(){return c("plugin:window|outer_size",{label:this.label}).then((({width:e,height:t})=>new b(e,t)))}async isFullscreen(){return c("plugin:window|is_fullscreen",{label:this.label})}async isMinimized(){return c("plugin:window|is_minimized",{label:this.label})}async isMaximized(){return c("plugin:window|is_maximized",{label:this.label})}async isFocused(){return c("plugin:window|is_focused",{label:this.label})}async isDecorated(){return c("plugin:window|is_decorated",{label:this.label})}async isResizable(){return c("plugin:window|is_resizable",{label:this.label})}async isMaximizable(){return c("plugin:window|is_maximizable",{label:this.label})}async isMinimizable(){return c("plugin:window|is_minimizable",{label:this.label})}async isClosable(){return c("plugin:window|is_closable",{label:this.label})}async isVisible(){return c("plugin:window|is_visible",{label:this.label})}async title(){return c("plugin:window|title",{label:this.label})}async theme(){return c("plugin:window|theme",{label:this.label})}async center(){return c("plugin:window|center",{label:this.label})}async requestUserAttention(e){let t=null;return e&&(t=e===P.Critical?{type:"Critical"}:{type:"Informational"}),c("plugin:window|request_user_attention",{label:this.label,value:t})}async setResizable(e){return c("plugin:window|set_resizable",{label:this.label,value:e})}async setMaximizable(e){return c("plugin:window|set_maximizable",{label:this.label,value:e})}async setMinimizable(e){return c("plugin:window|set_minimizable",{label:this.label,value:e})}async setClosable(e){return c("plugin:window|set_closable",{label:this.label,value:e})}async setTitle(e){return c("plugin:window|set_title",{label:this.label,value:e})}async maximize(){return c("plugin:window|maximize",{label:this.label})}async unmaximize(){return c("plugin:window|unmaximize",{label:this.label})}async toggleMaximize(){return c("plugin:window|toggle_maximize",{label:this.label})}async minimize(){return c("plugin:window|minimize",{label:this.label})}async unminimize(){return c("plugin:window|unminimize",{label:this.label})}async show(){return c("plugin:window|show",{label:this.label})}async hide(){return c("plugin:window|hide",{label:this.label})}async close(){return c("plugin:window|close",{label:this.label})}async destroy(){return c("plugin:window|destroy",{label:this.label})}async setDecorations(e){return c("plugin:window|set_decorations",{label:this.label,value:e})}async setShadow(e){return c("plugin:window|set_shadow",{label:this.label,value:e})}async setEffects(e){return c("plugin:window|set_effects",{label:this.label,value:e})}async clearEffects(){return c("plugin:window|set_effects",{label:this.label,value:null})}async setAlwaysOnTop(e){return c("plugin:window|set_always_on_top",{label:this.label,value:e})}async setAlwaysOnBottom(e){return c("plugin:window|set_always_on_bottom",{label:this.label,value:e})}async setContentProtected(e){return c("plugin:window|set_content_protected",{label:this.label,value:e})}async setSize(e){if(!e||"Logical"!==e.type&&"Physical"!==e.type)throw new Error("the `size` argument must be either a LogicalSize or a PhysicalSize instance");const t={};return t[`${e.type}`]={width:e.width,height:e.height},c("plugin:window|set_size",{label:this.label,value:t})}async setMinSize(e){if(e&&"Logical"!==e.type&&"Physical"!==e.type)throw new Error("the `size` argument must be either a LogicalSize or a PhysicalSize instance");let t=null;return e&&(t={},t[`${e.type}`]={width:e.width,height:e.height}),c("plugin:window|set_min_size",{label:this.label,value:t})}async setMaxSize(e){if(e&&"Logical"!==e.type&&"Physical"!==e.type)throw new Error("the `size` argument must be either a LogicalSize or a PhysicalSize instance");let t=null;return e&&(t={},t[`${e.type}`]={width:e.width,height:e.height}),c("plugin:window|set_max_size",{label:this.label,value:t})}async setPosition(e){if(!e||"Logical"!==e.type&&"Physical"!==e.type)throw new Error("the `position` argument must be either a LogicalPosition or a PhysicalPosition instance");const t={};return t[`${e.type}`]={x:e.x,y:e.y},c("plugin:window|set_position",{label:this.label,value:t})}async setFullscreen(e){return c("plugin:window|set_fullscreen",{label:this.label,value:e})}async setFocus(){return c("plugin:window|set_focus",{label:this.label})}async setIcon(e){return c("plugin:window|set_icon",{label:this.label,value:w(e)})}async setSkipTaskbar(e){return c("plugin:window|set_skip_taskbar",{label:this.label,value:e})}async setCursorGrab(e){return c("plugin:window|set_cursor_grab",{label:this.label,value:e})}async setCursorVisible(e){return c("plugin:window|set_cursor_visible",{label:this.label,value:e})}async setCursorIcon(e){return c("plugin:window|set_cursor_icon",{label:this.label,value:e})}async setCursorPosition(e){if(!e||"Logical"!==e.type&&"Physical"!==e.type)throw new Error("the `position` argument must be either a LogicalPosition or a PhysicalPosition instance");const t={};return t[`${e.type}`]={x:e.x,y:e.y},c("plugin:window|set_cursor_position",{label:this.label,value:t})}async setIgnoreCursorEvents(e){return c("plugin:window|set_ignore_cursor_events",{label:this.label,value:e})}async startDragging(){return c("plugin:window|start_dragging",{label:this.label})}async startResizeDragging(e){return c("plugin:window|start_resize_dragging",{label:this.label,value:e})}async setProgressBar(e){return c("plugin:window|set_progress_bar",{label:this.label,value:e})}async setVisibleOnAllWorkspaces(e){return c("plugin:window|set_visible_on_all_workspaces",{label:this.label,value:e})}async setTitleBarStyle(e){return c("plugin:window|set_title_bar_style",{label:this.label,value:e})}async onResized(e){return this.listen(f.WINDOW_RESIZED,(t=>{t.payload=J(t.payload),e(t)}))}async onMoved(e){return this.listen(f.WINDOW_MOVED,(t=>{t.payload=Z(t.payload),e(t)}))}async onCloseRequested(e){return this.listen(f.WINDOW_CLOSE_REQUESTED,(async t=>{const n=new B(t);await e(n),n.isPreventDefault()||await this.destroy()}))}async onDragDropEvent(e){const t=await this.listen(f.DRAG,(t=>{e({...t,payload:{type:"dragged",paths:t.payload.paths,position:Z(t.payload.position)}})})),n=await this.listen(f.DROP,(t=>{e({...t,payload:{type:"dropped",paths:t.payload.paths,position:Z(t.payload.position)}})})),i=await this.listen(f.DROP_OVER,(t=>{e({...t,payload:{type:"dragOver",position:Z(t.payload.position)}})})),r=await this.listen(f.DROP_CANCELLED,(t=>{e({...t,payload:{type:"cancelled"}})}));return()=>{t(),n(),i(),r()}}async onFocusChanged(e){const t=await this.listen(f.WINDOW_FOCUS,(t=>{e({...t,payload:!0})})),n=await this.listen(f.WINDOW_BLUR,(t=>{e({...t,payload:!1})}));return()=>{t(),n()}}async onScaleChanged(e){return this.listen(f.WINDOW_SCALE_FACTOR_CHANGED,e)}async onThemeChanged(e){return this.listen(f.WINDOW_THEME_CHANGED,e)}}var $,q;function Q(e){return null===e?null:{name:e.name,scaleFactor:e.scaleFactor,position:Z(e.position),size:J(e.size)}}function Z(e){return new v(e.x,e.y)}function J(e){return new b(e.width,e.height)}!function(e){e.AppearanceBased="appearanceBased",e.Light="light",e.Dark="dark",e.MediumLight="mediumLight",e.UltraDark="ultraDark",e.Titlebar="titlebar",e.Selection="selection",e.Menu="menu",e.Popover="popover",e.Sidebar="sidebar",e.HeaderView="headerView",e.Sheet="sheet",e.WindowBackground="windowBackground",e.HudWindow="hudWindow",e.FullScreenUI="fullScreenUI",e.Tooltip="tooltip",e.ContentBackground="contentBackground",e.UnderWindowBackground="underWindowBackground",e.UnderPageBackground="underPageBackground",e.Mica="mica",e.Blur="blur",e.Acrylic="acrylic",e.Tabbed="tabbed",e.TabbedDark="tabbedDark",e.TabbedLight="tabbedLight"}($||($={})),function(e){e.FollowsWindowActiveState="followsWindowActiveState",e.Active="active",e.Inactive="inactive"}(q||(q={}));var K=Object.freeze({__proto__:null,CloseRequestedEvent:B,get Effect(){return $},get EffectState(){return q},LogicalPosition:m,LogicalSize:g,PhysicalPosition:v,PhysicalSize:b,get ProgressBarStatus(){return C},get UserAttentionType(){return P},Window:H,availableMonitors:async function(){return c("plugin:window|available_monitors").then((e=>e.map(Q)))},currentMonitor:async function(){return c("plugin:window|current_monitor").then(Q)},cursorPosition:async function(){return c("plugin:window|cursor_position").then(Z)},getAllWindows:V,getCurrentWindow:j,monitorFromPoint:async function(e,t){return c("plugin:window|monitor_from_point",{x:e,y:t}).then(Q)},primaryMonitor:async function(){return c("plugin:window|primary_monitor").then(Q)}});function Y([e,t,n]){switch(n){case"Submenu":return new X(e,t);case"Predefined":return new M(e,t);case"Check":return new F(e,t);case"Icon":return new U(e,t);default:return new O(e,t)}}class X extends W{constructor(e,t){super(e,t,"Submenu")}static async new(e){return z("Submenu",e).then((([e,t])=>new X(e,t)))}async text(){return c("plugin:menu|text",{rid:this.rid,kind:this.kind})}async setText(e){return c("plugin:menu|set_text",{rid:this.rid,kind:this.kind,text:e})}async isEnabled(){return c("plugin:menu|is_enabled",{rid:this.rid,kind:this.kind})}async setEnabled(e){return c("plugin:menu|set_enabled",{rid:this.rid,kind:this.kind,enabled:e})}async append(e){return c("plugin:menu|append",{rid:this.rid,kind:this.kind,items:(Array.isArray(e)?e:[e]).map((e=>"rid"in e?[e.rid,e.kind]:e))})}async prepend(e){return c("plugin:menu|prepend",{rid:this.rid,kind:this.kind,items:(Array.isArray(e)?e:[e]).map((e=>"rid"in e?[e.rid,e.kind]:e))})}async insert(e,t){return c("plugin:menu|insert",{rid:this.rid,kind:this.kind,items:(Array.isArray(e)?e:[e]).map((e=>"rid"in e?[e.rid,e.kind]:e)),position:t})}async remove(e){return c("plugin:menu|remove",{rid:this.rid,kind:this.kind,item:[e.rid,e.kind]})}async removeAt(e){return c("plugin:menu|remove_at",{rid:this.rid,kind:this.kind,position:e}).then(Y)}async items(){return c("plugin:menu|items",{rid:this.rid,kind:this.kind}).then((e=>e.map(Y)))}async get(e){return c("plugin:menu|get",{rid:this.rid,kind:this.kind,id:e}).then((e=>e?Y(e):null))}async popup(e,t){var n;let i=null;return e&&(i={},i[""+(e instanceof v?"Physical":"Logical")]={x:e.x,y:e.y}),c("plugin:menu|popup",{rid:this.rid,kind:this.kind,window:null!==(n=null==t?void 0:t.label)&&void 0!==n?n:null,at:i})}async setAsWindowsMenuForNSApp(){return c("plugin:menu|set_as_windows_menu_for_nsapp",{rid:this.rid})}async setAsHelpMenuForNSApp(){return c("plugin:menu|set_as_help_menu_for_nsapp",{rid:this.rid})}}function ee([e,t,n]){switch(n){case"Submenu":return new X(e,t);case"Predefined":return new M(e,t);case"Check":return new F(e,t);case"Icon":return new U(e,t);default:return new O(e,t)}}class te extends W{constructor(e,t){super(e,t,"Menu")}static async new(e){return z("Menu",e).then((([e,t])=>new te(e,t)))}static async default(){return c("plugin:menu|create_default").then((([e,t])=>new te(e,t)))}async append(e){return c("plugin:menu|append",{rid:this.rid,kind:this.kind,items:(Array.isArray(e)?e:[e]).map((e=>"rid"in e?[e.rid,e.kind]:e))})}async prepend(e){return c("plugin:menu|prepend",{rid:this.rid,kind:this.kind,items:(Array.isArray(e)?e:[e]).map((e=>"rid"in e?[e.rid,e.kind]:e))})}async insert(e,t){return c("plugin:menu|insert",{rid:this.rid,kind:this.kind,items:(Array.isArray(e)?e:[e]).map((e=>"rid"in e?[e.rid,e.kind]:e)),position:t})}async remove(e){return c("plugin:menu|remove",{rid:this.rid,kind:this.kind,item:[e.rid,e.kind]})}async removeAt(e){return c("plugin:menu|remove_at",{rid:this.rid,kind:this.kind,position:e}).then(ee)}async items(){return c("plugin:menu|items",{rid:this.rid,kind:this.kind}).then((e=>e.map(ee)))}async get(e){return c("plugin:menu|get",{rid:this.rid,kind:this.kind,id:e}).then((e=>e?ee(e):null))}async popup(e,t){var n;let i=null;return e&&(i={},i[""+(e instanceof v?"Physical":"Logical")]={x:e.x,y:e.y}),c("plugin:menu|popup",{rid:this.rid,kind:this.kind,window:null!==(n=null==t?void 0:t.label)&&void 0!==n?n:null,at:i})}async setAsAppMenu(){return c("plugin:menu|set_as_app_menu",{rid:this.rid}).then((e=>e?new te(e[0],e[1]):null))}async setAsWindowMenu(e){var t;return c("plugin:menu|set_as_window_menu",{rid:this.rid,window:null!==(t=null==e?void 0:e.label)&&void 0!==t?t:null}).then((e=>e?new te(e[0],e[1]):null))}}var ne=Object.freeze({__proto__:null,CheckMenuItem:F,IconMenuItem:U,Menu:te,MenuItem:O,get NativeIcon(){return R},PredefinedMenuItem:M,Submenu:X});function ie(){var e;window.__TAURI_INTERNALS__=null!==(e=window.__TAURI_INTERNALS__)&&void 0!==e?e:{}}var re,ae=Object.freeze({__proto__:null,clearMocks:function(){var e,t,n;"object"==typeof window.__TAURI_INTERNALS__&&((null===(e=window.__TAURI_INTERNALS__)||void 0===e?void 0:e.convertFileSrc)&&delete window.__TAURI_INTERNALS__.convertFileSrc,(null===(t=window.__TAURI_INTERNALS__)||void 0===t?void 0:t.invoke)&&delete window.__TAURI_INTERNALS__.invoke,(null===(n=window.__TAURI_INTERNALS__)||void 0===n?void 0:n.metadata)&&delete window.__TAURI_INTERNALS__.metadata)},mockConvertFileSrc:function(e){ie(),window.__TAURI_INTERNALS__.convertFileSrc=function(t,n="asset"){const i=encodeURIComponent(t);return"windows"===e?`http://${n}.localhost/${i}`:`${n}://localhost/${i}`}},mockIPC:function(e){ie(),window.__TAURI_INTERNALS__.transformCallback=function(e,t=!1){const n=window.crypto.getRandomValues(new Uint32Array(1))[0],i=`_${n}`;return Object.defineProperty(window,i,{value:n=>(t&&Reflect.deleteProperty(window,i),e&&e(n)),writable:!1,configurable:!0}),n},window.__TAURI_INTERNALS__.invoke=function(t,n,i){return e(t,n)}},mockWindows:function(e,...t){ie(),window.__TAURI_INTERNALS__.metadata={windows:[e,...t].map((e=>({label:e}))),currentWindow:{label:e},webviews:[e,...t].map((e=>({windowLabel:e,label:e}))),currentWebview:{windowLabel:e,label:e}}}});!function(e){e[e.Audio=1]="Audio",e[e.Cache=2]="Cache",e[e.Config=3]="Config",e[e.Data=4]="Data",e[e.LocalData=5]="LocalData",e[e.Document=6]="Document",e[e.Download=7]="Download",e[e.Picture=8]="Picture",e[e.Public=9]="Public",e[e.Video=10]="Video",e[e.Resource=11]="Resource",e[e.Temp=12]="Temp",e[e.AppConfig=13]="AppConfig",e[e.AppData=14]="AppData",e[e.AppLocalData=15]="AppLocalData",e[e.AppCache=16]="AppCache",e[e.AppLog=17]="AppLog",e[e.Desktop=18]="Desktop",e[e.Executable=19]="Executable",e[e.Font=20]="Font",e[e.Home=21]="Home",e[e.Runtime=22]="Runtime",e[e.Template=23]="Template"}(re||(re={}));var se=Object.freeze({__proto__:null,get BaseDirectory(){return re},appCacheDir:async function(){return c("plugin:path|resolve_directory",{directory:re.AppCache})},appConfigDir:async function(){return c("plugin:path|resolve_directory",{directory:re.AppConfig})},appDataDir:async function(){return c("plugin:path|resolve_directory",{directory:re.AppData})},appLocalDataDir:async function(){return c("plugin:path|resolve_directory",{directory:re.AppLocalData})},appLogDir:async function(){return c("plugin:path|resolve_directory",{directory:re.AppLog})},audioDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Audio})},basename:async function(e,t){return c("plugin:path|basename",{path:e,ext:t})},cacheDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Cache})},configDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Config})},dataDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Data})},delimiter:function(){return window.__TAURI_INTERNALS__.plugins.path.delimiter},desktopDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Desktop})},dirname:async function(e){return c("plugin:path|dirname",{path:e})},documentDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Document})},downloadDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Download})},executableDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Executable})},extname:async function(e){return c("plugin:path|extname",{path:e})},fontDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Font})},homeDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Home})},isAbsolute:async function(e){return c("plugin:path|isAbsolute",{path:e})},join:async function(...e){return c("plugin:path|join",{paths:e})},localDataDir:async function(){return c("plugin:path|resolve_directory",{directory:re.LocalData})},normalize:async function(e){return c("plugin:path|normalize",{path:e})},pictureDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Picture})},publicDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Public})},resolve:async function(...e){return c("plugin:path|resolve",{paths:e})},resolveResource:async function(e){return c("plugin:path|resolve_directory",{directory:re.Resource,path:e})},resourceDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Resource})},runtimeDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Runtime})},sep:function(){return window.__TAURI_INTERNALS__.plugins.path.sep},tempDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Temp})},templateDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Template})},videoDir:async function(){return c("plugin:path|resolve_directory",{directory:re.Video})}});class le extends d{constructor(e,t){super(e),this.id=t}static async getById(e){return c("plugin:tray|get_by_id",{id:e}).then((t=>t?new le(t,e):null))}static async removeById(e){return c("plugin:tray|remove_by_id",{id:e})}static async new(e){(null==e?void 0:e.menu)&&(e.menu=[e.menu.rid,e.menu.kind]),(null==e?void 0:e.icon)&&(e.icon=w(e.icon));const t=new o;return(null==e?void 0:e.action)&&(t.onmessage=e.action,delete e.action),c("plugin:tray|new",{options:null!=e?e:{},handler:t}).then((([e,t])=>new le(e,t)))}async setIcon(e){let t=null;return e&&(t=w(e)),c("plugin:tray|set_icon",{rid:this.rid,icon:t})}async setMenu(e){return e&&(e=[e.rid,e.kind]),c("plugin:tray|set_menu",{rid:this.rid,menu:e})}async setTooltip(e){return c("plugin:tray|set_tooltip",{rid:this.rid,tooltip:e})}async setTitle(e){return c("plugin:tray|set_title",{rid:this.rid,title:e})}async setVisible(e){return c("plugin:tray|set_visible",{rid:this.rid,visible:e})}async setTempDirPath(e){return c("plugin:tray|set_temp_dir_path",{rid:this.rid,path:e})}async setIconAsTemplate(e){return c("plugin:tray|set_icon_as_template",{rid:this.rid,asTemplate:e})}async setMenuOnLeftClick(e){return c("plugin:tray|set_show_menu_on_left_click",{rid:this.rid,onLeft:e})}}var oe=Object.freeze({__proto__:null,TrayIcon:le});function ue(){return new pe(j(),window.__TAURI_INTERNALS__.metadata.currentWebview.label,{skip:!0})}function ce(){return window.__TAURI_INTERNALS__.metadata.webviews.map((e=>new pe(H.getByLabel(e.windowLabel),e.label,{skip:!0})))}const de=["tauri://created","tauri://error"];class pe{constructor(e,t,n){this.window=e,this.label=t,this.listeners=Object.create(null),(null==n?void 0:n.skip)||c("plugin:webview|create_webview",{windowLabel:e.label,label:t,options:n}).then((async()=>this.emit("tauri://created"))).catch((async e=>this.emit("tauri://error",e)))}static getByLabel(e){var t;return null!==(t=ce().find((t=>t.label===e)))&&void 0!==t?t:null}static getCurrent(){return ue()}static getAll(){return ce()}async listen(e,t){return this._handleTauriEvent(e,t)?()=>{const n=this.listeners[e];n.splice(n.indexOf(t),1)}:E(e,t,{target:{kind:"Webview",label:this.label}})}async once(e,t){return this._handleTauriEvent(e,t)?()=>{const n=this.listeners[e];n.splice(n.indexOf(t),1)}:D(e,t,{target:{kind:"Webview",label:this.label}})}async emit(e,t){if(!de.includes(e))return I(e,t);for(const n of this.listeners[e]||[])n({event:e,id:-1,payload:t})}async emitTo(e,t,n){if(!de.includes(t))return T(e,t,n);for(const e of this.listeners[t]||[])e({event:t,id:-1,payload:n})}_handleTauriEvent(e,t){return!!de.includes(e)&&(e in this.listeners?this.listeners[e].push(t):this.listeners[e]=[t],!0)}async position(){return c("plugin:webview|webview_position",{label:this.label}).then((({x:e,y:t})=>new v(e,t)))}async size(){return c("plugin:webview|webview_size",{label:this.label}).then((({width:e,height:t})=>new b(e,t)))}async close(){return c("plugin:webview|close",{label:this.label})}async setSize(e){if(!e||"Logical"!==e.type&&"Physical"!==e.type)throw new Error("the `size` argument must be either a LogicalSize or a PhysicalSize instance");const t={};return t[`${e.type}`]={width:e.width,height:e.height},c("plugin:webview|set_webview_size",{label:this.label,value:t})}async setPosition(e){if(!e||"Logical"!==e.type&&"Physical"!==e.type)throw new Error("the `position` argument must be either a LogicalPosition or a PhysicalPosition instance");const t={};return t[`${e.type}`]={x:e.x,y:e.y},c("plugin:webview|set_webview_position",{label:this.label,value:t})}async setFocus(){return c("plugin:webview|set_webview_focus",{label:this.label})}async setZoom(e){return c("plugin:webview|set_webview_zoom",{label:this.label,value:e})}async reparent(e){return c("plugin:webview|set_webview_focus",{label:this.label,window:"string"==typeof e?e:e.label})}async onDragDropEvent(e){const t=await this.listen(f.DRAG,(t=>{e({...t,payload:{type:"dragged",paths:t.payload.paths,position:he(t.payload.position)}})})),n=await this.listen(f.DROP,(t=>{e({...t,payload:{type:"dropped",paths:t.payload.paths,position:he(t.payload.position)}})})),i=await this.listen(f.DROP_CANCELLED,(t=>{e({...t,payload:{type:"dragOver",position:he(t.payload.position)}})})),r=await this.listen(f.DROP_CANCELLED,(t=>{e({...t,payload:{type:"cancelled"}})}));return()=>{t(),n(),i(),r()}}}function he(e){return new v(e.x,e.y)}var we,ye,_e=Object.freeze({__proto__:null,Webview:pe,getAllWebviews:ce,getCurrentWebview:ue});function ge(){const e=ue();return new me(e.label,{skip:!0})}function be(){return window.__TAURI_INTERNALS__.metadata.webviews.map((e=>new me(e.label,{skip:!0})))}class me{constructor(e,t={}){var n;this.label=e,this.listeners=Object.create(null),(null==t?void 0:t.skip)||c("plugin:webview|create_webview_window",{options:{...t,parent:"string"==typeof t.parent?t.parent:null===(n=t.parent)||void 0===n?void 0:n.label,label:e}}).then((async()=>this.emit("tauri://created"))).catch((async e=>this.emit("tauri://error",e)))}static getByLabel(e){var t;const n=null!==(t=be().find((t=>t.label===e)))&&void 0!==t?t:null;return n?new me(n.label,{skip:!0}):null}static getCurrent(){return ge()}static getAll(){return be().map((e=>new me(e.label,{skip:!0})))}async listen(e,t){return this._handleTauriEvent(e,t)?()=>{const n=this.listeners[e];n.splice(n.indexOf(t),1)}:E(e,t,{target:{kind:"WebviewWindow",label:this.label}})}async once(e,t){return this._handleTauriEvent(e,t)?()=>{const n=this.listeners[e];n.splice(n.indexOf(t),1)}:D(e,t,{target:{kind:"WebviewWindow",label:this.label}})}}we=me,ye=[H,pe],(Array.isArray(ye)?ye:[ye]).forEach((e=>{Object.getOwnPropertyNames(e.prototype).forEach((t=>{var n;"object"==typeof we.prototype&&we.prototype&&t in we.prototype||Object.defineProperty(we.prototype,t,null!==(n=Object.getOwnPropertyDescriptor(e.prototype,t))&&void 0!==n?n:Object.create(null))}))}));var ve=Object.freeze({__proto__:null,WebviewWindow:me,getAllWebviewWindows:be,getCurrentWebviewWindow:ge});return e.app=_,e.core=p,e.dpi=k,e.event=N,e.image=y,e.menu=ne,e.mocks=ae,e.path=se,e.tray=oe,e.webview=_e,e.webviewWindow=ve,e.window=K,e}({});window.__TAURI__=__TAURI_IIFE__;
