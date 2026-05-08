import{d as k,p as V,o,a as i,e as x,w as p,s as P,K as q,u as a,P as S,a2 as T,m as D,q as M,E as N,F as E,Q as C,R as W,U as h,V as w,b as t,f as y,W as f,H as c,X as _,r as m,k as F}from"./index.js";import{_ as I}from"./Table.vue_vue_type_script_setup_true_lang.js";var H=k({inheritAttrs:!1,__name:"AspectRatio",props:{ratio:{type:Number,required:!1,default:1},asChild:{type:Boolean,required:!1},as:{type:null,required:!1}},setup(b){const s=b,{forwardRef:u}=V(),r=D(()=>1/s.ratio*100);return(n,d)=>(o(),i("div",{style:T(`position: relative; width: 100%; padding-bottom: ${r.value}%`),"data-reka-aspect-ratio-wrapper":""},[x(a(S),q({ref:a(u),"as-child":n.asChild,as:n.as,style:{position:"absolute",inset:"0px"}},n.$attrs),{default:p(()=>[P(n.$slots,"default",{aspect:r.value})]),_:3},16,["as-child","as"])],4))}}),K=H;const v=k({__name:"AspectRatio",props:{ratio:{},asChild:{type:Boolean},as:{}},setup(b){const s=b;return(u,r)=>(o(),M(a(K),N(E(s)),{default:p(()=>[P(u.$slots,"default")]),_:3},16))}}),Q={class:"flex flex-col h-screen"},U={class:"flex flex-col"},X={class:"relative rounded-lg border overflow-hidden"},G={class:"flex items-center justify-between px-4 py-3 bg-zinc-100 dark:bg-zinc-800 border-b"},J={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},L={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},O={class:"rounded-lg border overflow-hidden"},Y={class:"flex items-center justify-center p-4 min-h-[100px] bg-zinc-100 dark:bg-zinc-900"},Z={class:"w-[450px] bg-muted rounded-md overflow-hidden border"},tt={class:"border-t"},et={key:0,class:"border-t"},ot={class:"flex items-center justify-between bg-zinc-100 dark:bg-zinc-800"},st={class:"flex"},rt={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},it={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},lt={class:"overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"},nt={class:"rounded-lg border overflow-hidden"},dt={class:"flex items-center justify-center p-4 min-h-[100px] bg-zinc-100 dark:bg-zinc-900"},at={class:"flex flex-row flex-wrap"},ut={class:"flex flex-col w-[150px]"},ct={class:"bg-muted rounded-md overflow-hidden"},pt={class:"flex flex-col w-[150px]"},xt={class:"bg-muted rounded-md overflow-hidden"},ft={class:"flex flex-col w-[150px]"},mt={class:"bg-muted rounded-md overflow-hidden"},vt={class:"flex flex-col w-[150px]"},bt={class:"bg-muted rounded-md overflow-hidden"},gt={class:"border-t"},ht={key:0,class:"border-t"},wt={class:"flex items-center justify-between bg-zinc-100 dark:bg-zinc-800"},yt={class:"flex"},kt={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},zt={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},Ct={class:"overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"},j=`div (style: "w-[450px] bg-muted rounded-md overflow-hidden border") {
    aspect-ratio (ratio: 1.777) {
        img (src: "https://images.unsplash.com/photo-1588345921523-c2dcdb7f1dcd?w=800&dpr=2&q=80", style: "object-cover w-full h-full", alt: "Photo by Drew Beamer") {}
    }
}
`,A=`<div class="w-[450px] bg-muted rounded-md overflow-hidden border">
  <AspectRatio :ratio="1.777">
    <img class="object-cover w-full h-full" src="https://images.unsplash.com/photo-1588345921523-c2dcdb7f1dcd?w=800&dpr=2&q=80" alt="Photo by Drew Beamer" />
  </AspectRatio>
</div>
`,R=`row (gap: "4", style: "flex-wrap") {
    col (gap: "2", style: "w-[150px]") {
        div (style: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 1) {
                div (style: "flex items-center justify-center w-full h-full bg-primary/10") {
                    text (text: "1:1") {}
                }
            }
        }
        text (text: "Square (1:1)", style: "text-sm text-muted-foreground") {}
    }
    col (gap: "2", style: "w-[150px]") {
        div (style: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 1.777) {
                div (style: "flex items-center justify-center w-full h-full bg-primary/10") {
                    text (text: "16:9") {}
                }
            }
        }
        text (style: "text-sm text-muted-foreground", text: "Widescreen (16:9)") {}
    }
    col (style: "w-[150px]", gap: "2") {
        div (style: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 0.5625) {
                div (style: "flex items-center justify-center w-full h-full bg-primary/10") {
                    text (text: "9:16") {}
                }
            }
        }
        text (style: "text-sm text-muted-foreground", text: "Portrait (9:16)") {}
    }
    col (gap: "2", style: "w-[150px]") {
        div (style: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 1.333) {
                div (style: "flex items-center justify-center w-full h-full bg-primary/10") {
                    text (text: "4:3") {}
                }
            }
        }
        text (text: "Standard (4:3)", style: "text-sm text-muted-foreground") {}
    }
}
`,B=`<div class="flex flex-row flex-wrap">
  <div class="flex flex-col w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio :ratio="1">
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          <span>1:1</span>
        </div>
      </AspectRatio>
    </div>
    <span class="text-sm text-muted-foreground">Square (1:1)</span>
  </div>
  <div class="flex flex-col w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio :ratio="1.777">
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          <span>16:9</span>
        </div>
      </AspectRatio>
    </div>
    <span class="text-sm text-muted-foreground">Widescreen (16:9)</span>
  </div>
  <div class="flex flex-col w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio :ratio="0.5625">
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          <span>9:16</span>
        </div>
      </AspectRatio>
    </div>
    <span class="text-sm text-muted-foreground">Portrait (9:16)</span>
  </div>
  <div class="flex flex-col w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio :ratio="1.333">
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          <span>4:3</span>
        </div>
      </AspectRatio>
    </div>
    <span class="text-sm text-muted-foreground">Standard (4:3)</span>
  </div>
</div>
`,$="npx shadcn-vue@latest add aspect-ratio",_t=k({__name:"aspectratio",setup(b){const s=m(""),u=m(!0),r=m("auto"),n=m(!0),d=m("auto");async function g(z,e){try{await navigator.clipboard.writeText(z),s.value=e,setTimeout(()=>{s.value=""},2e3)}catch(l){console.error("Failed to copy:",l)}}return C(r,()=>{h(()=>w.highlightAll())}),C(d,()=>{h(()=>w.highlightAll())}),W(()=>{h(()=>w.highlightAll())}),(z,e)=>(o(),i("div",Q,[t("div",U,[e[30]||(e[30]=t("h1",{class:"text-4xl font-bold tracking-tight"},"AspectRatio",-1)),e[31]||(e[31]=t("span",null,"Displays content within a desired ratio.",-1)),e[32]||(e[32]=t("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Installation",-1)),t("div",X,[t("div",G,[e[11]||(e[11]=t("span",{class:"text-xs text-zinc-600 dark:text-zinc-400 font-medium"},"bash",-1)),t("button",{onClick:e[0]||(e[0]=l=>g($,"codeblock1")),class:"inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[s.value!=="codeblock1"?(o(),i("svg",J,[...e[9]||(e[9]=[t("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),t("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(o(),i("svg",L,[...e[10]||(e[10]=[t("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),y(" "+f(s.value==="codeblock1"?"Copied!":"Copy"),1)])]),t("pre",{class:"p-4 text-sm bg-zinc-950 text-zinc-50 overflow-x-auto"},[t("code",{class:"block font-mono !p-0 language-bash"},f($))])]),e[33]||(e[33]=t("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Example",-1)),t("div",O,[t("div",Y,[t("div",Z,[x(a(v),{ratio:1.777},{default:p(()=>[...e[12]||(e[12]=[t("img",{class:"object-cover w-full h-full",src:"https://images.unsplash.com/photo-1588345921523-c2dcdb7f1dcd?w=800&dpr=2&q=80",alt:"Photo by Drew Beamer"},null,-1)])]),_:1})])]),t("div",tt,[t("button",{onClick:e[1]||(e[1]=l=>u.value=!u.value),class:"flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"},[e[14]||(e[14]=t("span",{class:"font-medium"},"Code",-1)),(o(),i("svg",{class:c([u.value?"rotate-180":"","transition-transform duration-200"]),xmlns:"http://www.w3.org/2000/svg",width:"16",height:"16",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[...e[13]||(e[13]=[t("path",{d:"m6 9 6 6 6-6"},null,-1)])],2))]),u.value?(o(),i("div",et,[t("div",ot,[t("div",st,[t("button",{onClick:e[2]||(e[2]=l=>r.value="auto"),class:c([r.value==="auto"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])}," Auto ",2),t("button",{onClick:e[3]||(e[3]=l=>r.value="vue"),class:c([r.value==="vue"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])}," Vue ",2)]),t("button",{onClick:e[4]||(e[4]=l=>g(r.value==="auto"?j:A,"aspectratio-basic")),class:"inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[s.value!=="aspectratio-basic"?(o(),i("svg",rt,[...e[15]||(e[15]=[t("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),t("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(o(),i("svg",it,[...e[16]||(e[16]=[t("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),y(" "+f(s.value==="aspectratio-basic"?"Copied!":"Copy"),1)])]),t("pre",lt,[t("code",{class:c("block font-mono !p-0 language-"+(r.value==="auto"?"auto":"html"))},f(r.value==="auto"?j:A),3)])])):_("",!0)])]),e[34]||(e[34]=t("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Common Ratios",-1)),t("div",nt,[t("div",dt,[t("div",at,[t("div",ut,[t("div",ct,[x(a(v),{ratio:1},{default:p(()=>[...e[17]||(e[17]=[t("div",{class:"flex items-center justify-center w-full h-full bg-primary/10"},[t("span",null,"1:1")],-1)])]),_:1})]),e[18]||(e[18]=t("span",{class:"text-sm text-muted-foreground"},"Square (1:1)",-1))]),t("div",pt,[t("div",xt,[x(a(v),{ratio:1.777},{default:p(()=>[...e[19]||(e[19]=[t("div",{class:"flex items-center justify-center w-full h-full bg-primary/10"},[t("span",null,"16:9")],-1)])]),_:1})]),e[20]||(e[20]=t("span",{class:"text-sm text-muted-foreground"},"Widescreen (16:9)",-1))]),t("div",ft,[t("div",mt,[x(a(v),{ratio:.5625},{default:p(()=>[...e[21]||(e[21]=[t("div",{class:"flex items-center justify-center w-full h-full bg-primary/10"},[t("span",null,"9:16")],-1)])]),_:1})]),e[22]||(e[22]=t("span",{class:"text-sm text-muted-foreground"},"Portrait (9:16)",-1))]),t("div",vt,[t("div",bt,[x(a(v),{ratio:1.333},{default:p(()=>[...e[23]||(e[23]=[t("div",{class:"flex items-center justify-center w-full h-full bg-primary/10"},[t("span",null,"4:3")],-1)])]),_:1})]),e[24]||(e[24]=t("span",{class:"text-sm text-muted-foreground"},"Standard (4:3)",-1))])])]),t("div",gt,[t("button",{onClick:e[5]||(e[5]=l=>n.value=!n.value),class:"flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"},[e[26]||(e[26]=t("span",{class:"font-medium"},"Code",-1)),(o(),i("svg",{class:c([n.value?"rotate-180":"","transition-transform duration-200"]),xmlns:"http://www.w3.org/2000/svg",width:"16",height:"16",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[...e[25]||(e[25]=[t("path",{d:"m6 9 6 6 6-6"},null,-1)])],2))]),n.value?(o(),i("div",ht,[t("div",wt,[t("div",yt,[t("button",{onClick:e[6]||(e[6]=l=>d.value="auto"),class:c([d.value==="auto"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])}," Auto ",2),t("button",{onClick:e[7]||(e[7]=l=>d.value="vue"),class:c([d.value==="vue"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])}," Vue ",2)]),t("button",{onClick:e[8]||(e[8]=l=>g(d.value==="auto"?R:B,"aspectratio-ratios")),class:"inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[s.value!=="aspectratio-ratios"?(o(),i("svg",kt,[...e[27]||(e[27]=[t("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),t("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(o(),i("svg",zt,[...e[28]||(e[28]=[t("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),y(" "+f(s.value==="aspectratio-ratios"?"Copied!":"Copy"),1)])]),t("pre",Ct,[t("code",{class:c("block font-mono !p-0 language-"+(d.value==="auto"?"auto":"html"))},f(d.value==="auto"?R:B),3)])])):_("",!0)])]),e[35]||(e[35]=t("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Properties",-1)),x(a(I),null,{default:p(()=>[...e[29]||(e[29]=[t("thead",{class:"bg-muted/50"},[t("tr",null,[t("th",{class:"border px-4 py-2 text-left font-semibold"},"Property"),t("th",{class:"border px-4 py-2 text-left font-semibold"},"Type"),t("th",{class:"border px-4 py-2 text-left font-semibold"},"Default"),t("th",{class:"border px-4 py-2 text-left font-semibold"},"Values"),t("th",{class:"border px-4 py-2 text-left font-semibold"},"Description")])],-1),t("tbody",null,[t("tr",null,[t("td",{class:"border px-4 py-2"},"ratio"),t("td",{class:"border px-4 py-2"},"number"),t("td",{class:"border px-4 py-2"},"1"),t("td",{class:"border px-4 py-2"},"-"),t("td",{class:"border px-4 py-2"},"The aspect ratio (width/height)")])],-1)])]),_:1})])]))}}),Rt=F(_t,[["__scopeId","data-v-bc0ee082"]]);export{Rt as default};
