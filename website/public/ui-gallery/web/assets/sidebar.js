import{d as C,Q as _,R as M,U as g,V as h,o as i,a as d,b as t,f as k,W as b,e as o,w as r,u as s,H as p,X as B,r as x,k as $}from"./index.js";import{_ as V,a as w,b as c,c as m,d as j,e as I}from"./index7.js";import{_ as D}from"./SidebarFooter.vue_vue_type_script_setup_true_lang.js";import{_ as F}from"./Table.vue_vue_type_script_setup_true_lang.js";import"./index6.js";import"./DialogClose.js";import"./DialogPortal.js";import"./x.js";import"./TooltipTrigger.vue_vue_type_script_setup_true_lang.js";import"./VisuallyHidden.js";import"./index4.js";const H={class:"flex flex-col h-screen"},T={class:"flex flex-col"},A={class:"relative rounded-lg border overflow-hidden"},N={class:"flex items-center justify-between px-4 py-3 bg-zinc-100 dark:bg-zinc-800 border-b"},P={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},W={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},E={class:"rounded-lg border overflow-hidden"},Q={class:"flex items-center justify-center p-4 min-h-[100px] bg-zinc-100 dark:bg-zinc-900"},R={class:"border-t"},U={key:0,class:"border-t"},X={class:"flex items-center justify-between bg-zinc-100 dark:bg-zinc-800"},q={class:"flex"},G={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},J={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},K={class:"overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"},y=`sidebar-provider {
    sidebar {
        sidebar-header {
            sidebar-menu {
                sidebar-menu-item {
                    sidebar-menu-button {
                        text (text: "Dashboard") {}
                    }
                }
            }
        }
        sidebar-content {
            sidebar-group {
                sidebar-group-label (text: "Main") {}
                sidebar-group-content {
                    sidebar-menu {
                        sidebar-menu-item {
                            sidebar-menu-button {
                                text (text: "Home") {}
                            }
                        }
                        sidebar-menu-item {
                            sidebar-menu-button {
                                text (text: "Settings") {}
                            }
                        }
                    }
                }
            }
        }
        sidebar-footer {
            text (text: "Footer") {}
        }
    }
}
`,z=`<div>
  <Sidebar>
    <SidebarHeader>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton>
            <span>Dashboard</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarHeader>
    <SidebarContent>
      <div>
        <div>Main</div>
        <div>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton>
                <span>Home</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
            <SidebarMenuItem>
              <SidebarMenuButton>
                <span>Settings</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </div>
      </div>
    </SidebarContent>
    <SidebarFooter>
      <span>Footer</span>
    </SidebarFooter>
  </Sidebar>
</div>
`,S="npx shadcn-vue@latest add sidebar",L=C({__name:"sidebar",setup(O){const a=x(""),u=x(!0),n=x("auto");async function f(v,e){try{await navigator.clipboard.writeText(v),a.value=e,setTimeout(()=>{a.value=""},2e3)}catch(l){console.error("Failed to copy:",l)}}return _(n,()=>{g(()=>h.highlightAll())}),M(()=>{g(()=>h.highlightAll())}),(v,e)=>(i(),d("div",H,[t("div",T,[e[18]||(e[18]=t("h1",{class:"text-4xl font-bold tracking-tight"},"Sidebar",-1)),e[19]||(e[19]=t("span",null,"A composable sidebar component with support for different sections and navigation items.",-1)),e[20]||(e[20]=t("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Installation",-1)),t("div",A,[t("div",N,[e[7]||(e[7]=t("span",{class:"text-xs text-zinc-600 dark:text-zinc-400 font-medium"},"bash",-1)),t("button",{onClick:e[0]||(e[0]=l=>f(S,"codeblock1")),class:"inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[a.value!=="codeblock1"?(i(),d("svg",P,[...e[5]||(e[5]=[t("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),t("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(i(),d("svg",W,[...e[6]||(e[6]=[t("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),k(" "+b(a.value==="codeblock1"?"Copied!":"Copy"),1)])]),t("pre",{class:"p-4 text-sm bg-zinc-950 text-zinc-50 overflow-x-auto"},[t("code",{class:"block font-mono !p-0 language-bash"},b(S))])]),e[21]||(e[21]=t("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Simple",-1)),t("div",E,[t("div",Q,[t("div",null,[o(s(I),null,{default:r(()=>[o(s(V),null,{default:r(()=>[o(s(w),null,{default:r(()=>[o(s(c),null,{default:r(()=>[o(s(m),null,{default:r(()=>[...e[8]||(e[8]=[t("span",null,"Dashboard",-1)])]),_:1})]),_:1})]),_:1})]),_:1}),o(s(j),null,{default:r(()=>[t("div",null,[e[11]||(e[11]=t("div",null,"Main",-1)),t("div",null,[o(s(w),null,{default:r(()=>[o(s(c),null,{default:r(()=>[o(s(m),null,{default:r(()=>[...e[9]||(e[9]=[t("span",null,"Home",-1)])]),_:1})]),_:1}),o(s(c),null,{default:r(()=>[o(s(m),null,{default:r(()=>[...e[10]||(e[10]=[t("span",null,"Settings",-1)])]),_:1})]),_:1})]),_:1})])])]),_:1}),o(s(D),null,{default:r(()=>[...e[12]||(e[12]=[t("span",null,"Footer",-1)])]),_:1})]),_:1})])]),t("div",R,[t("button",{onClick:e[1]||(e[1]=l=>u.value=!u.value),class:"flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"},[e[14]||(e[14]=t("span",{class:"font-medium"},"Code",-1)),(i(),d("svg",{class:p([u.value?"rotate-180":"","transition-transform duration-200"]),xmlns:"http://www.w3.org/2000/svg",width:"16",height:"16",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[...e[13]||(e[13]=[t("path",{d:"m6 9 6 6 6-6"},null,-1)])],2))]),u.value?(i(),d("div",U,[t("div",X,[t("div",q,[t("button",{onClick:e[2]||(e[2]=l=>n.value="auto"),class:p([n.value==="auto"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])}," Auto ",2),t("button",{onClick:e[3]||(e[3]=l=>n.value="vue"),class:p([n.value==="vue"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])}," Vue ",2)]),t("button",{onClick:e[4]||(e[4]=l=>f(n.value==="auto"?y:z,"sidebar-basic")),class:"inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[a.value!=="sidebar-basic"?(i(),d("svg",G,[...e[15]||(e[15]=[t("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),t("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(i(),d("svg",J,[...e[16]||(e[16]=[t("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),k(" "+b(a.value==="sidebar-basic"?"Copied!":"Copy"),1)])]),t("pre",K,[t("code",{class:p("block font-mono !p-0 language-"+(n.value==="auto"?"auto":"html"))},b(n.value==="auto"?y:z),3)])])):B("",!0)])]),e[22]||(e[22]=t("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Properties",-1)),o(s(F),null,{default:r(()=>[...e[17]||(e[17]=[t("thead",{class:"bg-muted/50"},[t("tr",null,[t("th",{class:"border px-4 py-2 text-left font-semibold"},"Property"),t("th",{class:"border px-4 py-2 text-left font-semibold"},"Type"),t("th",{class:"border px-4 py-2 text-left font-semibold"},"Default"),t("th",{class:"border px-4 py-2 text-left font-semibold"},"Values"),t("th",{class:"border px-4 py-2 text-left font-semibold"},"Description")])],-1),t("tbody",null,[t("tr",null,[t("td",{class:"border px-4 py-2"},"side"),t("td",{class:"border px-4 py-2"},"string"),t("td",{class:"border px-4 py-2"},'"left"'),t("td",{class:"border px-4 py-2"},'"left", "right"'),t("td",{class:"border px-4 py-2"},"Which side the sidebar appears")]),t("tr",null,[t("td",{class:"border px-4 py-2"},"variant"),t("td",{class:"border px-4 py-2"},"string"),t("td",{class:"border px-4 py-2"},'"sidebar"'),t("td",{class:"border px-4 py-2"},'"sidebar", "floating", "inset"'),t("td",{class:"border px-4 py-2"},"Sidebar style variant")]),t("tr",null,[t("td",{class:"border px-4 py-2"},"collapsible"),t("td",{class:"border px-4 py-2"},"string"),t("td",{class:"border px-4 py-2"},'"offcanvas"'),t("td",{class:"border px-4 py-2"},'"offcanvas", "icon", "none"'),t("td",{class:"border px-4 py-2"},"Collapsible behavior")])],-1)])]),_:1})])]))}}),lt=$(L,[["__scopeId","data-v-c02a95ed"]]);export{lt as default};
