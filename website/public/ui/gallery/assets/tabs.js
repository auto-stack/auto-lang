import{d as y,t as tt,m as at,n as S,k as nt,z as lt,o as u,p as z,w as n,q as _,u as s,P,r as w,bf as rt,s as it,Q as et,b1 as dt,e as a,a4 as ut,W as M,a8 as ct,l as B,aA as pt,x as ft,Z as bt,c as N,A as gt,D as vt,E as xt,F as R,J as F,H as D,K as mt,b as e,O as A,R as V,U as $,a as x,f as v,V as k,G as m,B as Y,_ as E,j as ht}from"./index.js";import{_ as I}from"./Card.vue_vue_type_script_setup_true_lang.js";import{_ as G}from"./Input.vue_vue_type_script_setup_true_lang.js";import{_ as wt}from"./Table.vue_vue_type_script_setup_true_lang.js";import{R as yt}from"./RovingFocusItem.js";import"./index4.js";const[O,kt]=it("TabsRoot");var Ct=y({__name:"TabsRoot",props:{defaultValue:{type:null,required:!1},orientation:{type:String,required:!1,default:"horizontal"},dir:{type:String,required:!1},activationMode:{type:String,required:!1,default:"automatic"},modelValue:{type:null,required:!1},unmountOnHide:{type:Boolean,required:!1,default:!0},asChild:{type:Boolean,required:!1},as:{type:null,required:!1}},emits:["update:modelValue"],setup(p,{emit:o}){const d=p,l=o,{orientation:g,unmountOnHide:c,dir:f}=tt(d),r=at(f);S();const b=nt(d,"modelValue",l,{defaultValue:d.defaultValue,passive:d.modelValue===void 0}),h=w(),t=rt(new Set);return kt({modelValue:b,changeModelValue:i=>{b.value=i},orientation:g,dir:r,unmountOnHide:c,activationMode:d.activationMode,baseId:lt(void 0,"reka-tabs"),tabsList:h,contentIds:t,registerContent:i=>{t.value=new Set([...t.value,i])},unregisterContent:i=>{const j=new Set(t.value);j.delete(i),t.value=j}}),(i,j)=>(u(),z(s(P),{dir:s(r),"data-orientation":s(g),"as-child":i.asChild,as:i.as},{default:n(()=>[_(i.$slots,"default",{modelValue:s(b)})]),_:3},8,["dir","data-orientation","as-child","as"]))}}),Tt=Ct;function st(p,o){return`${p}-trigger-${o}`}function ot(p,o){return`${p}-content-${o}`}var zt=y({__name:"TabsContent",props:{value:{type:[String,Number],required:!0},forceMount:{type:Boolean,required:!1},asChild:{type:Boolean,required:!1},as:{type:null,required:!1}},setup(p){const o=p,{forwardRef:d}=S(),l=O(),g=B(()=>st(l.baseId,o.value)),c=B(()=>ot(l.baseId,o.value)),f=B(()=>o.value===l.modelValue.value),r=w(f.value);return et(()=>{l.registerContent(o.value),requestAnimationFrame(()=>{r.value=!1})}),dt(()=>{l.unregisterContent(o.value)}),(b,h)=>(u(),z(s(ct),{present:b.forceMount||f.value,"force-mount":""},{default:n(({present:t})=>[a(s(P),{id:c.value,ref:s(d),"as-child":b.asChild,as:b.as,role:"tabpanel","data-state":f.value?"active":"inactive","data-orientation":s(l).orientation.value,"aria-labelledby":g.value,hidden:!t,tabindex:"0",style:ut({animationDuration:r.value?"0s":void 0})},{default:n(()=>[!s(l).unmountOnHide.value||t?_(b.$slots,"default",{key:0}):M("v-if",!0)]),_:2},1032,["id","as-child","as","data-state","data-orientation","aria-labelledby","hidden","style"])]),_:3},8,["present"]))}}),_t=zt,Bt=y({__name:"TabsList",props:{loop:{type:Boolean,required:!1,default:!0},asChild:{type:Boolean,required:!1},as:{type:null,required:!1}},setup(p){const o=p,{loop:d}=tt(o),{forwardRef:l,currentElement:g}=S(),c=O();return c.tabsList=g,(f,r)=>(u(),z(s(pt),{"as-child":"",orientation:s(c).orientation.value,dir:s(c).dir.value,loop:s(d)},{default:n(()=>[a(s(P),{ref:s(l),role:"tablist","as-child":f.asChild,as:f.as,"aria-orientation":s(c).orientation.value},{default:n(()=>[_(f.$slots,"default")]),_:3},8,["as-child","as","aria-orientation"])]),_:3},8,["orientation","dir","loop"]))}}),Vt=Bt,$t=y({__name:"TabsTrigger",props:{value:{type:[String,Number],required:!0},disabled:{type:Boolean,required:!1,default:!1},asChild:{type:Boolean,required:!1},as:{type:null,required:!1,default:"button"}},setup(p){const o=p,{forwardRef:d}=S(),l=O(),g=B(()=>st(l.baseId,o.value)),c=B(()=>l.contentIds.value.has(o.value)?ot(l.baseId,o.value):void 0),f=B(()=>o.value===l.modelValue.value);return(r,b)=>(u(),z(s(yt),{"as-child":"",focusable:!r.disabled,active:f.value},{default:n(()=>[a(s(P),{id:g.value,ref:s(d),role:"tab",type:r.as==="button"?"button":void 0,as:r.as,"as-child":r.asChild,"aria-selected":f.value?"true":"false","aria-controls":c.value,"data-state":f.value?"active":"inactive",disabled:r.disabled,"data-disabled":r.disabled?"":void 0,"data-orientation":s(l).orientation.value,onMousedown:b[0]||(b[0]=bt(h=>{!r.disabled&&h.ctrlKey===!1?s(l).changeModelValue(r.value):h.preventDefault()},["left"])),onKeydown:b[1]||(b[1]=ft(h=>s(l).changeModelValue(r.value),["enter","space"])),onFocus:b[2]||(b[2]=()=>{const h=s(l).activationMode!=="manual";!f.value&&!r.disabled&&h&&s(l).changeModelValue(r.value)})},{default:n(()=>[_(r.$slots,"default")]),_:3},8,["id","type","as","as-child","aria-selected","aria-controls","data-state","disabled","data-disabled","data-orientation"])]),_:3},8,["focusable","active"]))}}),Mt=$t;/**
 * @license lucide-vue-next v0.312.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const St=N("CreditCardIcon",[["rect",{width:"20",height:"14",x:"2",y:"5",rx:"2",key:"ynyp8z"}],["line",{x1:"2",x2:"22",y1:"10",y2:"10",key:"1b3vmo"}]]);/**
 * @license lucide-vue-next v0.312.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const Pt=N("LockIcon",[["rect",{width:"18",height:"11",x:"3",y:"11",rx:"2",ry:"2",key:"1w4ew1"}],["path",{d:"M7 11V7a5 5 0 0 1 10 0v4",key:"fwvmzm"}]]);/**
 * @license lucide-vue-next v0.312.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const U=N("UserIcon",[["path",{d:"M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2",key:"975kel"}],["circle",{cx:"12",cy:"7",r:"4",key:"17ys0d"}]]),q=y({__name:"Tabs",props:{defaultValue:{},orientation:{},dir:{},activationMode:{},modelValue:{},unmountOnHide:{type:Boolean},asChild:{type:Boolean},as:{}},emits:["update:modelValue"],setup(p,{emit:o}){const g=gt(p,o);return(c,f)=>(u(),z(s(Tt),vt(xt(s(g))),{default:n(()=>[_(c.$slots,"default")]),_:3},16))}}),C=y({__name:"TabsContent",props:{value:{},forceMount:{type:Boolean},asChild:{type:Boolean},as:{},class:{type:[Boolean,null,String,Object,Array]}},setup(p){const o=p,d=R(o,"class");return(l,g)=>(u(),z(s(_t),F({class:s(D)("mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",o.class)},s(d)),{default:n(()=>[_(l.$slots,"default")]),_:3},16,["class"]))}}),L=y({__name:"TabsList",props:{loop:{type:Boolean},asChild:{type:Boolean},as:{},class:{type:[Boolean,null,String,Object,Array]}},setup(p){const o=p,d=R(o,"class");return(l,g)=>(u(),z(s(Vt),F(s(d),{class:s(D)("inline-flex items-center justify-center rounded-md bg-muted p-1 text-muted-foreground",o.class)}),{default:n(()=>[_(l.$slots,"default")]),_:3},16,["class"]))}}),jt={class:"truncate"},T=y({__name:"TabsTrigger",props:{value:{},disabled:{type:Boolean},asChild:{type:Boolean},as:{},class:{type:[Boolean,null,String,Object,Array]}},setup(p){const o=p,d=R(o,"class"),l=mt(d);return(g,c)=>(u(),z(s(Mt),F(s(l),{class:s(D)("inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm",o.class)}),{default:n(()=>[e("span",jt,[_(g.$slots,"default")])]),_:3},16,["class"]))}}),At={class:"flex flex-col pb-8"},It={class:"flex flex-col"},qt={class:"relative rounded-lg border overflow-hidden"},Lt={class:"flex items-center justify-between px-4 py-3 bg-zinc-100 dark:bg-zinc-800 border-b"},Nt={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},Rt={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},Ft={class:"rounded-lg border overflow-hidden"},Dt={class:"flex items-center justify-center p-6 min-h-[100px]"},Ot={class:"border-t"},Yt={key:0,class:"border-t"},Et={class:"flex items-center justify-between bg-zinc-100 dark:bg-zinc-800"},Gt={class:"flex"},Ut={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},Ht={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},Kt={class:"overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"},Wt={class:"rounded-lg border overflow-hidden mt-4"},Jt={class:"flex items-center justify-center p-6 min-h-[100px]"},Qt={class:"border-t"},Zt={key:0,class:"border-t"},Xt={class:"flex items-center justify-between bg-zinc-100 dark:bg-zinc-800"},te={class:"flex"},ee={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},se={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},oe={class:"overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"},ae={class:"rounded-lg border overflow-hidden mt-4"},ne={class:"flex items-center justify-center p-6 min-h-[100px]"},le={class:"flex flex-col gap-4"},re={class:"grid grid-cols-2 gap-4"},ie={class:"flex flex-col gap-2"},de={class:"flex flex-col gap-2"},ue={class:"flex flex-col gap-4"},ce={class:"border-t"},pe={key:0,class:"border-t"},fe={class:"flex items-center justify-between bg-zinc-100 dark:bg-zinc-800"},be={class:"flex"},ge={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},ve={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},xe={class:"overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"},H=`tabs (style: "w-full", default: "account") {
    tabs-list {
        tabs-trigger (text: "Account", value: "account") {}
        tabs-trigger (value: "password", text: "Password") {}
    }
    tabs-content (value: "account") {
        text (text: "Make changes to your account here.") {}
    }
    tabs-content (value: "password") {
        text (text: "Change your password here.") {}
    }
}
`,K=`<Tabs default-value="account" class="w-full">
  <TabsList>
    <TabsTrigger value="account">Account</TabsTrigger>
    <TabsTrigger value="password">Password</TabsTrigger>
  </TabsList>
  <TabsContent value="account">
    <span>Make changes to your account here.</span>
  </TabsContent>
  <TabsContent value="password">
    <span>Change your password here.</span>
  </TabsContent>
</Tabs>
`,W=`tabs (style: "w-full", default: "profile") {
    tabs-list {
        tabs-trigger (value: "profile") {
            icon (name: "user", style: "mr-2 h-4 w-4") {}
            text (text: "Profile") {}
        }
        tabs-trigger (value: "security") {
            icon (name: "lock", style: "mr-2 h-4 w-4") {}
            text (text: "Security") {}
        }
        tabs-trigger (value: "notifications") {
            icon (name: "bell", style: "mr-2 h-4 w-4") {}
            text (text: "Notifications") {}
        }
    }
    tabs-content (value: "profile") {
        text (text: "Manage your public profile information.") {}
    }
    tabs-content (value: "security") {
        text (text: "Configure security settings and 2FA.") {}
    }
    tabs-content (value: "notifications") {
        text (text: "Choose what notifications you receive.") {}
    }
}
`,J=`<Tabs default-value="profile" class="w-full">
  <TabsList>
    <TabsTrigger value="profile">
      <User class="mr-2 h-4 w-4" />
      Profile
    </TabsTrigger>
    <TabsTrigger value="security">
      <Lock class="mr-2 h-4 w-4" />
      Security
    </TabsTrigger>
    <TabsTrigger value="notifications">
      <Bell class="mr-2 h-4 w-4" />
      Notifications
    </TabsTrigger>
  </TabsList>
  <TabsContent value="profile">
    <span>Manage your public profile information.</span>
  </TabsContent>
  <TabsContent value="security">
    <span>Configure security settings and 2FA.</span>
  </TabsContent>
  <TabsContent value="notifications">
    <span>Choose what notifications you receive.</span>
  </TabsContent>
</Tabs>
`,Q=`tabs (style: "w-full", default: "general") {
    tabs-list {
        tabs-trigger (text: "General", value: "general") {}
        tabs-trigger (text: "Billing", value: "billing") {}
        tabs-trigger (text: "Notifications", value: "notifications") {}
    }
    tabs-content (value: "general") {
        card (style: "p-6") {
            col (gap: "4") {
                text (text: "General Settings", style: "text-lg font-semibold") {}
                row (gap: "4") {
                    col (gap: "2", style: "flex-1") {
                        label (text: "Name") {}
                        input (placeholder: "Your name") {}
                    }
                    col (gap: "2", style: "flex-1") {
                        label (text: "Email") {}
                        input (placeholder: "Your email") {}
                    }
                }
                button (text: "Save") {}
            }
        }
    }
    tabs-content (value: "billing") {
        card (style: "p-6") {
            col (gap: "4") {
                text (text: "Billing & Plans", style: "text-lg font-semibold") {}
                text (text: "You are currently on the free plan.") {}
                button (text: "Upgrade to Pro") {}
            }
        }
    }
    tabs-content (value: "notifications") {
        card (style: "p-6") {
            col (gap: "4") {
                text (text: "Notification Preferences", style: "text-lg font-semibold") {}
                text (text: "Configure how you receive notifications.") {}
            }
        }
    }
}
`,Z=`<Tabs default-value="general" class="w-full">
  <TabsList class="grid w-full grid-cols-3">
    <TabsTrigger value="general">General</TabsTrigger>
    <TabsTrigger value="billing">Billing</TabsTrigger>
    <TabsTrigger value="notifications">Notifications</TabsTrigger>
  </TabsList>
  <TabsContent value="general">
    <Card class="p-6">
      <div class="flex flex-col gap-4">
        <h3 class="text-lg font-semibold">General Settings</h3>
        <div class="grid grid-cols-2 gap-4">
          <div class="flex flex-col gap-2">
            <Label>Name</Label>
            <Input placeholder="Your name" />
          </div>
          <div class="flex flex-col gap-2">
            <Label>Email</Label>
            <Input placeholder="Your email" />
          </div>
        </div>
        <Button>Save Changes</Button>
      </div>
    </Card>
  </TabsContent>
  <TabsContent value="billing">
    <Card class="p-6">
      <div class="flex flex-col gap-4">
        <h3 class="text-lg font-semibold">Billing & Plans</h3>
        <p class="text-sm text-muted-foreground">
          You are currently on the <strong>Free</strong> plan.
        </p>
        <Button>Upgrade to Pro</Button>
      </div>
    </Card>
  </TabsContent>
  <TabsContent value="notifications">
    <Card class="p-6">
      <div class="flex flex-col gap-4">
        <h3 class="text-lg font-semibold">Notification Preferences</h3>
        <p class="text-sm text-muted-foreground">
          Configure how and when you receive notifications.
        </p>
      </div>
    </Card>
  </TabsContent>
</Tabs>
`,X="npx shadcn-vue@latest add tabs",me=y({__name:"tabs",setup(p){const o=w(""),d=w(!0),l=w("auto"),g=w(!0),c=w("auto"),f=w(!0),r=w("auto");async function b(h,t){try{await navigator.clipboard.writeText(h),o.value=t,setTimeout(()=>{o.value=""},2e3)}catch(i){console.error("Failed to copy:",i)}}return A(l,()=>{V(()=>$.highlightAll())}),A(c,()=>{V(()=>$.highlightAll())}),A(r,()=>{V(()=>$.highlightAll())}),et(()=>{V(()=>$.highlightAll())}),(h,t)=>(u(),x("div",At,[e("div",It,[t[50]||(t[50]=e("h1",{class:"text-4xl font-bold tracking-tight"},"Tabs",-1)),t[51]||(t[51]=e("span",{class:"text-muted-foreground"},"A set of layered sections of content, known as tab panels, that display one panel of content at a time.",-1)),t[52]||(t[52]=e("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Installation",-1)),e("div",qt,[e("div",Lt,[t[15]||(t[15]=e("span",{class:"text-xs text-zinc-600 dark:text-zinc-400 font-medium"},"bash",-1)),e("button",{onClick:t[0]||(t[0]=i=>b(X,"codeblock1")),class:"inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[o.value!=="codeblock1"?(u(),x("svg",Nt,[...t[13]||(t[13]=[e("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),e("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(u(),x("svg",Rt,[...t[14]||(t[14]=[e("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),v(" "+k(o.value==="codeblock1"?"Copied!":"Copy"),1)])]),e("pre",{class:"p-4 text-sm bg-zinc-950 text-zinc-50 overflow-x-auto"},[e("code",{class:"block font-mono !p-0 language-bash"},k(X))])]),t[53]||(t[53]=e("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Simple",-1)),e("div",Ft,[e("div",Dt,[a(s(q),{"default-value":"account",class:"w-full max-w-md"},{default:n(()=>[a(s(L),null,{default:n(()=>[a(s(T),{value:"account"},{default:n(()=>[...t[16]||(t[16]=[v("Account",-1)])]),_:1}),a(s(T),{value:"password"},{default:n(()=>[...t[17]||(t[17]=[v("Password",-1)])]),_:1})]),_:1}),a(s(C),{value:"account"},{default:n(()=>[...t[18]||(t[18]=[e("span",{class:"text-sm text-muted-foreground"},"Make changes to your account here. Click save when you're done.",-1)])]),_:1}),a(s(C),{value:"password"},{default:n(()=>[...t[19]||(t[19]=[e("span",{class:"text-sm text-muted-foreground"},"Change your password here. After saving, you'll be logged out.",-1)])]),_:1})]),_:1})]),e("div",Ot,[e("button",{onClick:t[1]||(t[1]=i=>d.value=!d.value),class:"flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"},[t[21]||(t[21]=e("span",{class:"font-medium"},"Code",-1)),(u(),x("svg",{class:m([d.value?"rotate-180":"","transition-transform duration-200"]),xmlns:"http://www.w3.org/2000/svg",width:"16",height:"16",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[...t[20]||(t[20]=[e("path",{d:"m6 9 6 6 6-6"},null,-1)])],2))]),d.value?(u(),x("div",Yt,[e("div",Et,[e("div",Gt,[e("button",{onClick:t[2]||(t[2]=i=>l.value="auto"),class:m([l.value==="auto"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Auto",2),e("button",{onClick:t[3]||(t[3]=i=>l.value="vue"),class:m([l.value==="vue"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Vue",2)]),e("button",{onClick:t[4]||(t[4]=i=>b(l.value==="auto"?H:K,"tabs-basic")),class:"inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[o.value!=="tabs-basic"?(u(),x("svg",Ut,[...t[22]||(t[22]=[e("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),e("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(u(),x("svg",Ht,[...t[23]||(t[23]=[e("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),v(" "+k(o.value==="tabs-basic"?"Copied!":"Copy"),1)])]),e("pre",Kt,[e("code",{class:m("block font-mono !p-0 language-"+(l.value==="auto"?"auto":"html"))},k(l.value==="auto"?H:K),3)])])):M("",!0)])]),t[54]||(t[54]=e("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"With Icons",-1)),t[55]||(t[55]=e("span",{class:"text-sm text-muted-foreground mt-1"},"Tab triggers with icons for better visual context.",-1)),e("div",Wt,[e("div",Jt,[a(s(q),{"default-value":"profile",class:"w-full max-w-md"},{default:n(()=>[a(s(L),null,{default:n(()=>[a(s(T),{value:"profile",class:"gap-2"},{default:n(()=>[a(s(U),{class:"h-4 w-4"}),t[24]||(t[24]=v(" Profile ",-1))]),_:1}),a(s(T),{value:"security",class:"gap-2"},{default:n(()=>[a(s(Pt),{class:"h-4 w-4"}),t[25]||(t[25]=v(" Security ",-1))]),_:1}),a(s(T),{value:"notifications",class:"gap-2"},{default:n(()=>[a(s(Y),{class:"h-4 w-4"}),t[26]||(t[26]=v(" Alerts ",-1))]),_:1})]),_:1}),a(s(C),{value:"profile"},{default:n(()=>[...t[27]||(t[27]=[e("span",{class:"text-sm text-muted-foreground"},"Manage your public profile information and bio.",-1)])]),_:1}),a(s(C),{value:"security"},{default:n(()=>[...t[28]||(t[28]=[e("span",{class:"text-sm text-muted-foreground"},"Configure two-factor authentication and security keys.",-1)])]),_:1}),a(s(C),{value:"notifications"},{default:n(()=>[...t[29]||(t[29]=[e("span",{class:"text-sm text-muted-foreground"},"Choose what notifications you want to receive.",-1)])]),_:1})]),_:1})]),e("div",Qt,[e("button",{onClick:t[5]||(t[5]=i=>g.value=!g.value),class:"flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"},[t[31]||(t[31]=e("span",{class:"font-medium"},"Code",-1)),(u(),x("svg",{class:m([g.value?"rotate-180":"","transition-transform duration-200"]),xmlns:"http://www.w3.org/2000/svg",width:"16",height:"16",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[...t[30]||(t[30]=[e("path",{d:"m6 9 6 6 6-6"},null,-1)])],2))]),g.value?(u(),x("div",Zt,[e("div",Xt,[e("div",te,[e("button",{onClick:t[6]||(t[6]=i=>c.value="auto"),class:m([c.value==="auto"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Auto",2),e("button",{onClick:t[7]||(t[7]=i=>c.value="vue"),class:m([c.value==="vue"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Vue",2)]),e("button",{onClick:t[8]||(t[8]=i=>b(c.value==="auto"?W:J,"tabs-icon")),class:"inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[o.value!=="tabs-icon"?(u(),x("svg",ee,[...t[32]||(t[32]=[e("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),e("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(u(),x("svg",se,[...t[33]||(t[33]=[e("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),v(" "+k(o.value==="tabs-icon"?"Copied!":"Copy"),1)])]),e("pre",oe,[e("code",{class:m("block font-mono !p-0 language-"+(c.value==="auto"?"auto":"html"))},k(c.value==="auto"?W:J),3)])])):M("",!0)])]),t[56]||(t[56]=e("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Settings",-1)),t[57]||(t[57]=e("span",{class:"text-sm text-muted-foreground mt-1"},"A full-width settings-style tab layout with form content.",-1)),e("div",ae,[e("div",ne,[a(s(q),{"default-value":"general",class:"w-full"},{default:n(()=>[a(s(L),{class:"grid w-full grid-cols-3"},{default:n(()=>[a(s(T),{value:"general",class:"gap-2"},{default:n(()=>[a(s(U),{class:"h-4 w-4"}),t[34]||(t[34]=v(" General ",-1))]),_:1}),a(s(T),{value:"billing",class:"gap-2"},{default:n(()=>[a(s(St),{class:"h-4 w-4"}),t[35]||(t[35]=v(" Billing ",-1))]),_:1}),a(s(T),{value:"notifications",class:"gap-2"},{default:n(()=>[a(s(Y),{class:"h-4 w-4"}),t[36]||(t[36]=v(" Notifications ",-1))]),_:1})]),_:1}),a(s(C),{value:"general"},{default:n(()=>[a(s(I),{class:"p-6"},{default:n(()=>[e("div",le,[t[40]||(t[40]=e("h3",{class:"text-lg font-semibold"},"General Settings",-1)),e("div",re,[e("div",ie,[t[37]||(t[37]=e("label",{class:"text-sm font-medium"},"Name",-1)),a(s(G),{placeholder:"Your name"})]),e("div",de,[t[38]||(t[38]=e("label",{class:"text-sm font-medium"},"Email",-1)),a(s(G),{placeholder:"Your email"})])]),a(s(E),{class:"w-fit"},{default:n(()=>[...t[39]||(t[39]=[v("Save Changes",-1)])]),_:1})])]),_:1})]),_:1}),a(s(C),{value:"billing"},{default:n(()=>[a(s(I),{class:"p-6"},{default:n(()=>[e("div",ue,[t[42]||(t[42]=e("h3",{class:"text-lg font-semibold"},"Billing & Plans",-1)),t[43]||(t[43]=e("p",{class:"text-sm text-muted-foreground"},[v(" You are currently on the "),e("strong",null,"Free"),v(" plan. ")],-1)),a(s(E),{class:"w-fit"},{default:n(()=>[...t[41]||(t[41]=[v("Upgrade to Pro",-1)])]),_:1})])]),_:1})]),_:1}),a(s(C),{value:"notifications"},{default:n(()=>[a(s(I),{class:"p-6"},{default:n(()=>[...t[44]||(t[44]=[e("div",{class:"flex flex-col gap-4"},[e("h3",{class:"text-lg font-semibold"},"Notification Preferences"),e("p",{class:"text-sm text-muted-foreground"}," Configure how and when you receive notifications. ")],-1)])]),_:1})]),_:1})]),_:1})]),e("div",ce,[e("button",{onClick:t[9]||(t[9]=i=>f.value=!f.value),class:"flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"},[t[46]||(t[46]=e("span",{class:"font-medium"},"Code",-1)),(u(),x("svg",{class:m([f.value?"rotate-180":"","transition-transform duration-200"]),xmlns:"http://www.w3.org/2000/svg",width:"16",height:"16",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[...t[45]||(t[45]=[e("path",{d:"m6 9 6 6 6-6"},null,-1)])],2))]),f.value?(u(),x("div",pe,[e("div",fe,[e("div",be,[e("button",{onClick:t[10]||(t[10]=i=>r.value="auto"),class:m([r.value==="auto"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Auto",2),e("button",{onClick:t[11]||(t[11]=i=>r.value="vue"),class:m([r.value==="vue"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Vue",2)]),e("button",{onClick:t[12]||(t[12]=i=>b(r.value==="auto"?Q:Z,"tabs-settings")),class:"inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[o.value!=="tabs-settings"?(u(),x("svg",ge,[...t[47]||(t[47]=[e("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),e("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(u(),x("svg",ve,[...t[48]||(t[48]=[e("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),v(" "+k(o.value==="tabs-settings"?"Copied!":"Copy"),1)])]),e("pre",xe,[e("code",{class:m("block font-mono !p-0 language-"+(r.value==="auto"?"auto":"html"))},k(r.value==="auto"?Q:Z),3)])])):M("",!0)])]),t[58]||(t[58]=e("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Properties",-1)),a(s(wt),null,{default:n(()=>[...t[49]||(t[49]=[e("thead",{class:"bg-muted/50"},[e("tr",null,[e("th",{class:"border px-4 py-2 text-left font-semibold"},"Property"),e("th",{class:"border px-4 py-2 text-left font-semibold"},"Type"),e("th",{class:"border px-4 py-2 text-left font-semibold"},"Default"),e("th",{class:"border px-4 py-2 text-left font-semibold"},"Values"),e("th",{class:"border px-4 py-2 text-left font-semibold"},"Description")])],-1),e("tbody",null,[e("tr",null,[e("td",{class:"border px-4 py-2"},"default"),e("td",{class:"border px-4 py-2"},"string"),e("td",{class:"border px-4 py-2"},"-"),e("td",{class:"border px-4 py-2"},"-"),e("td",{class:"border px-4 py-2"},"Default active tab value")]),e("tr",null,[e("td",{class:"border px-4 py-2"},"value"),e("td",{class:"border px-4 py-2"},"string"),e("td",{class:"border px-4 py-2"},"-"),e("td",{class:"border px-4 py-2"},"-"),e("td",{class:"border px-4 py-2"},"Tab trigger/content value")])],-1)])]),_:1})])]))}}),ze=ht(me,[["__scopeId","data-v-0def991a"]]);export{ze as default};
