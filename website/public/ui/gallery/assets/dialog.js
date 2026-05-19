import{d as v,A as Y,o as l,p as D,w as i,q as h,D as V,E as O,u as o,a2 as et,F as N,e as s,a0 as ot,$ as it,J as H,H as z,b as e,K as Z,a as u,G as g,O as B,Q as st,R as _,U as q,f as d,V as b,_ as y,W as $,ao as at,r as k,j as lt}from"./index.js";import{D as tt}from"./DialogClose.js";import{D as rt}from"./DialogPortal.js";import{X as nt}from"./x.js";import{D as dt,a as ut,b as ct}from"./DialogTrigger.js";import{_ as U}from"./Input.vue_vue_type_script_setup_true_lang.js";import{_ as I}from"./Label.vue_vue_type_script_setup_true_lang.js";import{_ as pt}from"./Table.vue_vue_type_script_setup_true_lang.js";import"./index4.js";const j=v({__name:"Dialog",props:{open:{type:Boolean},defaultOpen:{type:Boolean},modal:{type:Boolean}},emits:["update:open"],setup(c,{emit:a}){const x=Y(c,a);return(p,w)=>(l(),D(o(et),V(O(o(x))),{default:i(()=>[h(p.$slots,"default")]),_:3},16))}}),P=v({__name:"DialogClose",props:{asChild:{type:Boolean},as:{}},setup(c){const a=c;return(r,n)=>(l(),D(o(tt),V(O(a)),{default:i(()=>[h(r.$slots,"default")]),_:3},16))}}),S=v({__name:"DialogContent",props:{forceMount:{type:Boolean},disableOutsidePointerEvents:{type:Boolean},asChild:{type:Boolean},as:{},class:{type:[Boolean,null,String,Object,Array]}},emits:["escapeKeyDown","pointerDownOutside","focusOutside","interactOutside","openAutoFocus","closeAutoFocus"],setup(c,{emit:a}){const r=c,n=a,x=N(r,"class"),p=Y(x,n);return(w,f)=>(l(),D(o(rt),null,{default:i(()=>[s(o(ot),{class:"fixed inset-0 z-50 bg-black/80 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"}),s(o(it),H(o(p),{class:o(z)("fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border bg-background p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg",r.class)}),{default:i(()=>[h(w.$slots,"default"),s(o(tt),{class:"absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted-foreground"},{default:i(()=>[s(o(nt),{class:"w-4 h-4"}),f[0]||(f[0]=e("span",{class:"sr-only"},"Close",-1))]),_:1})]),_:3},16,["class"])]),_:3}))}}),T=v({__name:"DialogDescription",props:{asChild:{type:Boolean},as:{},class:{type:[Boolean,null,String,Object,Array]}},setup(c){const a=c,r=N(a,"class"),n=Z(r);return(x,p)=>(l(),D(o(dt),H(o(n),{class:o(z)("text-sm text-muted-foreground",a.class)}),{default:i(()=>[h(x.$slots,"default")]),_:3},16,["class"]))}}),A=v({__name:"DialogFooter",props:{class:{type:[Boolean,null,String,Object,Array]}},setup(c){const a=c;return(r,n)=>(l(),u("div",{class:g(o(z)("flex flex-col-reverse sm:flex-row sm:justify-end sm:gap-x-2",a.class))},[h(r.$slots,"default")],2))}}),F=v({__name:"DialogHeader",props:{class:{type:[Boolean,null,String,Object,Array]}},setup(c){const a=c;return(r,n)=>(l(),u("div",{class:g(o(z)("flex flex-col gap-y-1.5 text-center sm:text-left",a.class))},[h(r.$slots,"default")],2))}}),E=v({__name:"DialogTitle",props:{asChild:{type:Boolean},as:{},class:{type:[Boolean,null,String,Object,Array]}},setup(c){const a=c,r=N(a,"class"),n=Z(r);return(x,p)=>(l(),D(o(ut),H(o(n),{class:o(z)("text-lg font-semibold leading-none tracking-tight",a.class)}),{default:i(()=>[h(x.$slots,"default")]),_:3},16,["class"]))}}),M=v({__name:"DialogTrigger",props:{asChild:{type:Boolean},as:{}},setup(c){const a=c;return(r,n)=>(l(),D(o(ct),V(O(a)),{default:i(()=>[h(r.$slots,"default")]),_:3},16))}}),mt={class:"flex flex-col pb-8"},gt={class:"flex flex-col"},xt={class:"relative rounded-lg border overflow-hidden"},ft={class:"flex items-center justify-between px-4 py-3 bg-zinc-100 dark:bg-zinc-800 border-b"},vt={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},bt={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},ht={class:"rounded-lg border overflow-hidden"},wt={class:"flex items-center justify-center p-6 min-h-[100px]"},kt={class:"border-t"},yt={key:0,class:"border-t"},Dt={class:"flex items-center justify-between bg-zinc-100 dark:bg-zinc-800"},zt={class:"flex"},Ct={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},_t={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},qt={class:"overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"},Bt={class:"rounded-lg border overflow-hidden mt-4"},$t={class:"flex items-center justify-center p-6 min-h-[100px]"},jt={class:"grid gap-4 py-4"},Pt={class:"grid grid-cols-4 items-center gap-4"},St={class:"grid grid-cols-4 items-center gap-4"},Tt={class:"border-t"},At={key:0,class:"border-t"},Ft={class:"flex items-center justify-between bg-zinc-100 dark:bg-zinc-800"},Et={class:"flex"},Mt={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},Vt={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},Ot={class:"overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"},Nt={class:"rounded-lg border overflow-hidden mt-4"},Ht={class:"flex items-center justify-center p-6 min-h-[100px]"},Lt={class:"border-t"},Ut={key:0,class:"border-t"},It={class:"flex items-center justify-between bg-zinc-100 dark:bg-zinc-800"},Rt={class:"flex"},Jt={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},Kt={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},Wt={class:"overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"},R=`row (gap: "4") {
    dialog {
        dialog-trigger (text: "Open Dialog") {}
        dialog-content {
            dialog-header {
                dialog-title (text: "Edit Profile") {}
                dialog-description (text: "Make changes to your profile here. Click save when you're done.") {}
            }
            dialog-footer {
                dialog-close (text: "Save changes") {}
            }
        }
    }
}
`,J=`<Dialog>
  <DialogTrigger as-child>
    <Button variant="outline">Edit Profile</Button>
  </DialogTrigger>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Edit Profile</DialogTitle>
      <DialogDescription>
        Make changes to your profile here. Click save when you're done.
      </DialogDescription>
    </DialogHeader>
    <DialogFooter>
      <DialogClose as-child>
        <Button type="submit">Save changes</Button>
      </DialogClose>
    </DialogFooter>
  </DialogContent>
</Dialog>
`,K=`dialog {
    dialog-trigger (text: "Edit Profile") {}
    dialog-content {
        dialog-header {
            dialog-title (text: "Edit Profile") {}
            dialog-description (text: "Make changes to your profile here. Click save when you're done.") {}
        }
        col (gap: "4", style: "py-4") {
            row (gap: "4") {
                col (style: "flex-1", gap: "2") {
                    label (text: "First name") {}
                    input (placeholder: "John") {}
                }
                col (style: "flex-1", gap: "2") {
                    label (text: "Last name") {}
                    input (placeholder: "Doe") {}
                }
            }
            col (gap: "2") {
                label (text: "Username") {}
                input (placeholder: "@johndoe") {}
            }
        }
        dialog-footer {
            dialog-close (text: "Save changes") {}
        }
    }
}
`,W=`<Dialog>
  <DialogTrigger as-child>
    <Button variant="outline">Edit Profile</Button>
  </DialogTrigger>
  <DialogContent class="sm:max-w-[425px]">
    <DialogHeader>
      <DialogTitle>Edit Profile</DialogTitle>
      <DialogDescription>
        Make changes to your profile here. Click save when you're done.
      </DialogDescription>
    </DialogHeader>
    <div class="grid gap-4 py-4">
      <div class="grid grid-cols-4 items-center gap-4">
        <Label for="name" class="text-right">Name</Label>
        <Input id="name" default-value="John Doe" class="col-span-3" />
      </div>
      <div class="grid grid-cols-4 items-center gap-4">
        <Label for="username" class="text-right">Username</Label>
        <Input id="username" default-value="@johndoe" class="col-span-3" />
      </div>
    </div>
    <DialogFooter>
      <DialogClose as-child>
        <Button type="submit">Save changes</Button>
      </DialogClose>
    </DialogFooter>
  </DialogContent>
</Dialog>
`,G=`dialog {
    dialog-trigger (text: "Scrollable") {}
    dialog-content {
        dialog-header {
            dialog-title (text: "Terms of Service") {}
            dialog-description (text: "Please read the terms carefully.") {}
        }
        scroll-area (style: "max-h-[300px] pr-4") {
            text (text: "Lorem ipsum dolor sit amet...") {}
        }
        dialog-footer {
            dialog-close (text: "I Accept") {}
        }
    }
}
`,Q=`<Dialog>
  <DialogTrigger as-child>
    <Button variant="outline">Read Terms</Button>
  </DialogTrigger>
  <DialogContent class="sm:max-w-[500px]">
    <DialogHeader>
      <DialogTitle>Terms of Service</DialogTitle>
      <DialogDescription>
        Please read the terms of service carefully before accepting.
      </DialogDescription>
    </DialogHeader>
    <ScrollArea class="max-h-[300px] pr-4">
      <div class="text-sm text-muted-foreground space-y-4">
        <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.</p>
        <p>Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</p>
        <p>Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo.</p>
        <p>Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt.</p>
        <p>Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem.</p>
        <p>Ut enim ad minima veniam, quis nostrum exercitationem ullam corporis suscipit laboriosam, nisi ut aliquid ex ea commodi consequatur.</p>
      </div>
    </ScrollArea>
    <DialogFooter>
      <DialogClose as-child>
        <Button type="submit">I Accept</Button>
      </DialogClose>
    </DialogFooter>
  </DialogContent>
</Dialog>
`,X="npx shadcn-vue@latest add dialog",Gt=v({__name:"dialog",setup(c){const a=k(""),r=k(!0),n=k("auto"),x=k(!0),p=k("auto"),w=k(!0),f=k("auto");async function C(L,t){try{await navigator.clipboard.writeText(L),a.value=t,setTimeout(()=>{a.value=""},2e3)}catch(m){console.error("Failed to copy:",m)}}return B(n,()=>{_(()=>q.highlightAll())}),B(p,()=>{_(()=>q.highlightAll())}),B(f,()=>{_(()=>q.highlightAll())}),st(()=>{_(()=>q.highlightAll())}),(L,t)=>(l(),u("div",mt,[e("div",gt,[t[44]||(t[44]=e("h1",{class:"text-4xl font-bold tracking-tight"},"Dialog",-1)),t[45]||(t[45]=e("span",{class:"text-muted-foreground"},"A modal dialog that displays content on top of the page.",-1)),t[46]||(t[46]=e("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Installation",-1)),e("div",xt,[e("div",ft,[t[15]||(t[15]=e("span",{class:"text-xs text-zinc-600 dark:text-zinc-400 font-medium"},"bash",-1)),e("button",{onClick:t[0]||(t[0]=m=>C(X,"codeblock1")),class:"inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[a.value!=="codeblock1"?(l(),u("svg",vt,[...t[13]||(t[13]=[e("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),e("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(l(),u("svg",bt,[...t[14]||(t[14]=[e("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),d(" "+b(a.value==="codeblock1"?"Copied!":"Copy"),1)])]),e("pre",{class:"p-4 text-sm bg-zinc-950 text-zinc-50 overflow-x-auto"},[e("code",{class:"block font-mono !p-0 language-bash"},b(X))])]),t[47]||(t[47]=e("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Simple",-1)),e("div",ht,[e("div",wt,[s(o(j),null,{default:i(()=>[s(o(M),{"as-child":""},{default:i(()=>[s(o(y),{variant:"outline"},{default:i(()=>[...t[16]||(t[16]=[d("Edit Profile",-1)])]),_:1})]),_:1}),s(o(S),null,{default:i(()=>[s(o(F),null,{default:i(()=>[s(o(E),null,{default:i(()=>[...t[17]||(t[17]=[d("Edit Profile",-1)])]),_:1}),s(o(T),null,{default:i(()=>[...t[18]||(t[18]=[d("Make changes to your profile here. Click save when you're done.",-1)])]),_:1})]),_:1}),s(o(A),null,{default:i(()=>[s(o(P),{"as-child":""},{default:i(()=>[s(o(y),{type:"submit"},{default:i(()=>[...t[19]||(t[19]=[d("Save changes",-1)])]),_:1})]),_:1})]),_:1})]),_:1})]),_:1})]),e("div",kt,[e("button",{onClick:t[1]||(t[1]=m=>r.value=!r.value),class:"flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"},[t[21]||(t[21]=e("span",{class:"font-medium"},"Code",-1)),(l(),u("svg",{class:g([r.value?"rotate-180":"","transition-transform duration-200"]),xmlns:"http://www.w3.org/2000/svg",width:"16",height:"16",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[...t[20]||(t[20]=[e("path",{d:"m6 9 6 6 6-6"},null,-1)])],2))]),r.value?(l(),u("div",yt,[e("div",Dt,[e("div",zt,[e("button",{onClick:t[2]||(t[2]=m=>n.value="auto"),class:g([n.value==="auto"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Auto",2),e("button",{onClick:t[3]||(t[3]=m=>n.value="vue"),class:g([n.value==="vue"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Vue",2)]),e("button",{onClick:t[4]||(t[4]=m=>C(n.value==="auto"?R:J,"dialog-basic")),class:"inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[a.value!=="dialog-basic"?(l(),u("svg",Ct,[...t[22]||(t[22]=[e("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),e("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(l(),u("svg",_t,[...t[23]||(t[23]=[e("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),d(" "+b(a.value==="dialog-basic"?"Copied!":"Copy"),1)])]),e("pre",qt,[e("code",{class:g("block font-mono !p-0 language-"+(n.value==="auto"?"auto":"html"))},b(n.value==="auto"?R:J),3)])])):$("",!0)])]),t[48]||(t[48]=e("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"With Form",-1)),t[49]||(t[49]=e("span",{class:"text-sm text-muted-foreground mt-1"},"A dialog with form inputs for user data entry.",-1)),e("div",Bt,[e("div",$t,[s(o(j),null,{default:i(()=>[s(o(M),{"as-child":""},{default:i(()=>[s(o(y),{variant:"outline"},{default:i(()=>[...t[24]||(t[24]=[d("Edit Profile",-1)])]),_:1})]),_:1}),s(o(S),{class:"sm:max-w-[425px]"},{default:i(()=>[s(o(F),null,{default:i(()=>[s(o(E),null,{default:i(()=>[...t[25]||(t[25]=[d("Edit Profile",-1)])]),_:1}),s(o(T),null,{default:i(()=>[...t[26]||(t[26]=[d(" Make changes to your profile here. Click save when you're done. ",-1)])]),_:1})]),_:1}),e("div",jt,[e("div",Pt,[s(o(I),{for:"name",class:"text-right"},{default:i(()=>[...t[27]||(t[27]=[d("Name",-1)])]),_:1}),s(o(U),{id:"name","default-value":"John Doe",class:"col-span-3"})]),e("div",St,[s(o(I),{for:"username",class:"text-right"},{default:i(()=>[...t[28]||(t[28]=[d("Username",-1)])]),_:1}),s(o(U),{id:"username","default-value":"@johndoe",class:"col-span-3"})])]),s(o(A),null,{default:i(()=>[s(o(P),{"as-child":""},{default:i(()=>[s(o(y),{type:"submit"},{default:i(()=>[...t[29]||(t[29]=[d("Save changes",-1)])]),_:1})]),_:1})]),_:1})]),_:1})]),_:1})]),e("div",Tt,[e("button",{onClick:t[5]||(t[5]=m=>x.value=!x.value),class:"flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"},[t[31]||(t[31]=e("span",{class:"font-medium"},"Code",-1)),(l(),u("svg",{class:g([x.value?"rotate-180":"","transition-transform duration-200"]),xmlns:"http://www.w3.org/2000/svg",width:"16",height:"16",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[...t[30]||(t[30]=[e("path",{d:"m6 9 6 6 6-6"},null,-1)])],2))]),x.value?(l(),u("div",At,[e("div",Ft,[e("div",Et,[e("button",{onClick:t[6]||(t[6]=m=>p.value="auto"),class:g([p.value==="auto"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Auto",2),e("button",{onClick:t[7]||(t[7]=m=>p.value="vue"),class:g([p.value==="vue"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Vue",2)]),e("button",{onClick:t[8]||(t[8]=m=>C(p.value==="auto"?K:W,"dialog-form")),class:"inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[a.value!=="dialog-form"?(l(),u("svg",Mt,[...t[32]||(t[32]=[e("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),e("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(l(),u("svg",Vt,[...t[33]||(t[33]=[e("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),d(" "+b(a.value==="dialog-form"?"Copied!":"Copy"),1)])]),e("pre",Ot,[e("code",{class:g("block font-mono !p-0 language-"+(p.value==="auto"?"auto":"html"))},b(p.value==="auto"?K:W),3)])])):$("",!0)])]),t[50]||(t[50]=e("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Scrollable",-1)),t[51]||(t[51]=e("span",{class:"text-sm text-muted-foreground mt-1"},"A dialog with scrollable content for longer text.",-1)),e("div",Nt,[e("div",Ht,[s(o(j),null,{default:i(()=>[s(o(M),{"as-child":""},{default:i(()=>[s(o(y),{variant:"outline"},{default:i(()=>[...t[34]||(t[34]=[d("Read Terms",-1)])]),_:1})]),_:1}),s(o(S),{class:"sm:max-w-[500px]"},{default:i(()=>[s(o(F),null,{default:i(()=>[s(o(E),null,{default:i(()=>[...t[35]||(t[35]=[d("Terms of Service",-1)])]),_:1}),s(o(T),null,{default:i(()=>[...t[36]||(t[36]=[d(" Please read the terms of service carefully before accepting. ",-1)])]),_:1})]),_:1}),s(o(at),{class:"max-h-[300px] pr-4"},{default:i(()=>[...t[37]||(t[37]=[e("div",{class:"text-sm text-muted-foreground space-y-4"},[e("p",null,"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat."),e("p",null,"Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."),e("p",null,"Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo."),e("p",null,"Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt."),e("p",null,"Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem."),e("p",null,"Ut enim ad minima veniam, quis nostrum exercitationem ullam corporis suscipit laboriosam, nisi ut aliquid ex ea commodi consequatur.")],-1)])]),_:1}),s(o(A),null,{default:i(()=>[s(o(P),{"as-child":""},{default:i(()=>[s(o(y),{type:"submit"},{default:i(()=>[...t[38]||(t[38]=[d("I Accept",-1)])]),_:1})]),_:1})]),_:1})]),_:1})]),_:1})]),e("div",Lt,[e("button",{onClick:t[9]||(t[9]=m=>w.value=!w.value),class:"flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"},[t[40]||(t[40]=e("span",{class:"font-medium"},"Code",-1)),(l(),u("svg",{class:g([w.value?"rotate-180":"","transition-transform duration-200"]),xmlns:"http://www.w3.org/2000/svg",width:"16",height:"16",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[...t[39]||(t[39]=[e("path",{d:"m6 9 6 6 6-6"},null,-1)])],2))]),w.value?(l(),u("div",Ut,[e("div",It,[e("div",Rt,[e("button",{onClick:t[10]||(t[10]=m=>f.value="auto"),class:g([f.value==="auto"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Auto",2),e("button",{onClick:t[11]||(t[11]=m=>f.value="vue"),class:g([f.value==="vue"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])},"Vue",2)]),e("button",{onClick:t[12]||(t[12]=m=>C(f.value==="auto"?G:Q,"dialog-scroll")),class:"inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[a.value!=="dialog-scroll"?(l(),u("svg",Jt,[...t[41]||(t[41]=[e("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),e("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(l(),u("svg",Kt,[...t[42]||(t[42]=[e("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),d(" "+b(a.value==="dialog-scroll"?"Copied!":"Copy"),1)])]),e("pre",Wt,[e("code",{class:g("block font-mono !p-0 language-"+(f.value==="auto"?"auto":"html"))},b(f.value==="auto"?G:Q),3)])])):$("",!0)])]),t[52]||(t[52]=e("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Properties",-1)),s(o(pt),null,{default:i(()=>[...t[43]||(t[43]=[e("thead",{class:"bg-muted/50"},[e("tr",null,[e("th",{class:"border px-4 py-2 text-left font-semibold"},"Property"),e("th",{class:"border px-4 py-2 text-left font-semibold"},"Type"),e("th",{class:"border px-4 py-2 text-left font-semibold"},"Default"),e("th",{class:"border px-4 py-2 text-left font-semibold"},"Values"),e("th",{class:"border px-4 py-2 text-left font-semibold"},"Description")])],-1),e("tbody",null,[e("tr",null,[e("td",{class:"border px-4 py-2"},"open"),e("td",{class:"border px-4 py-2"},"boolean"),e("td",{class:"border px-4 py-2"},"false"),e("td",{class:"border px-4 py-2"},"true, false"),e("td",{class:"border px-4 py-2"},"Controls dialog visibility")]),e("tr",null,[e("td",{class:"border px-4 py-2"},"title"),e("td",{class:"border px-4 py-2"},"string"),e("td",{class:"border px-4 py-2"},"-"),e("td",{class:"border px-4 py-2"},"-"),e("td",{class:"border px-4 py-2"},"Dialog title")]),e("tr",null,[e("td",{class:"border px-4 py-2"},"description"),e("td",{class:"border px-4 py-2"},"string"),e("td",{class:"border px-4 py-2"},"-"),e("td",{class:"border px-4 py-2"},"-"),e("td",{class:"border px-4 py-2"},"Dialog description")])],-1)])]),_:1})])]))}}),ae=lt(Gt,[["__scopeId","data-v-757ddb4b"]]);export{ae as default};
