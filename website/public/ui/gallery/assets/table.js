import{d as V,o as b,a as u,G as p,u as l,H as P,q as I,O as N,Q as B,R as v,U as y,b as t,f as s,V as m,e as a,w as o,W as R,r as C,j as H}from"./index.js";import{_ as k}from"./Table.vue_vue_type_script_setup_true_lang.js";import{_ as A,a as d,b as T,c as j,d as n}from"./TableHeader.vue_vue_type_script_setup_true_lang.js";const M=V({__name:"TableCaption",props:{class:{type:[Boolean,null,String,Object,Array]}},setup(g){const i=g;return(x,r)=>(b(),u("caption",{class:p(l(P)("mt-4 text-sm text-muted-foreground",i.class))},[I(x.$slots,"default")],2))}}),U={class:"flex flex-col pb-8"},S={class:"flex flex-col"},D={class:"relative rounded-lg border overflow-hidden"},E={class:"flex items-center justify-between px-4 py-3 bg-zinc-100 dark:bg-zinc-800 border-b"},O={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},q={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},F={class:"rounded-lg border overflow-hidden"},G={class:"flex items-center justify-center p-4 min-h-[100px] bg-zinc-100 dark:bg-zinc-900"},Q={class:"border-t"},W={key:0,class:"border-t"},Y={class:"flex items-center justify-between bg-zinc-100 dark:bg-zinc-800"},J={class:"flex"},K={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},L={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},X={class:"overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"},z=`table {
    table-caption (text: "A list of your recent invoices.") {}
    table-header {
        table-row {
            table-head (text: "Invoice", style: "w-[100px]") {}
            table-head (text: "Status") {}
            table-head (text: "Method") {}
            table-head (text: "Amount", style: "text-right") {}
        }
    }
    table-body {
        table-row {
            table-cell (text: "INV001", style: "font-medium") {}
            table-cell (text: "Paid") {}
            table-cell (text: "Credit Card") {}
            table-cell (style: "text-right", text: "$250.00") {}
        }
        table-row {
            table-cell (style: "font-medium", text: "INV002") {}
            table-cell (text: "Pending") {}
            table-cell (text: "PayPal") {}
            table-cell (text: "$150.00", style: "text-right") {}
        }
        table-row {
            table-cell (text: "INV003", style: "font-medium") {}
            table-cell (text: "Unpaid") {}
            table-cell (text: "Bank Transfer") {}
            table-cell (style: "text-right", text: "$350.00") {}
        }
        table-row {
            table-cell (text: "INV004", style: "font-medium") {}
            table-cell (text: "Paid") {}
            table-cell (text: "Credit Card") {}
            table-cell (style: "text-right", text: "$450.00") {}
        }
        table-row {
            table-cell (style: "font-medium", text: "INV005") {}
            table-cell (text: "Paid") {}
            table-cell (text: "PayPal") {}
            table-cell (text: "$550.00", style: "text-right") {}
        }
        table-row {
            table-cell (text: "INV006", style: "font-medium") {}
            table-cell (text: "Pending") {}
            table-cell (text: "Bank Transfer") {}
            table-cell (style: "text-right", text: "$200.00") {}
        }
        table-row {
            table-cell (text: "INV007", style: "font-medium") {}
            table-cell (text: "Unpaid") {}
            table-cell (text: "Credit Card") {}
            table-cell (text: "$300.00", style: "text-right") {}
        }
    }
}
`,$=`<Table>
  <TableCaption>A list of your recent invoices.</TableCaption>
  <TableHeader>
    <TableRow>
      <TableHead class="w-[100px]">Invoice</TableHead>
      <TableHead>Status</TableHead>
      <TableHead>Method</TableHead>
      <TableHead class="text-right">Amount</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    <TableRow>
      <TableCell class="font-medium">INV001</TableCell>
      <TableCell>Paid</TableCell>
      <TableCell>Credit Card</TableCell>
      <TableCell class="text-right">$250.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV002</TableCell>
      <TableCell>Pending</TableCell>
      <TableCell>PayPal</TableCell>
      <TableCell class="text-right">$150.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV003</TableCell>
      <TableCell>Unpaid</TableCell>
      <TableCell>Bank Transfer</TableCell>
      <TableCell class="text-right">$350.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV004</TableCell>
      <TableCell>Paid</TableCell>
      <TableCell>Credit Card</TableCell>
      <TableCell class="text-right">$450.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV005</TableCell>
      <TableCell>Paid</TableCell>
      <TableCell>PayPal</TableCell>
      <TableCell class="text-right">$550.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV006</TableCell>
      <TableCell>Pending</TableCell>
      <TableCell>Bank Transfer</TableCell>
      <TableCell class="text-right">$200.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV007</TableCell>
      <TableCell>Unpaid</TableCell>
      <TableCell>Credit Card</TableCell>
      <TableCell class="text-right">$300.00</TableCell>
    </TableRow>
  </TableBody>
</Table>
`,h="npx shadcn-vue@latest add table",Z=V({__name:"table",setup(g){const i=C(""),x=C(!0),r=C("auto");async function w(c,e){try{await navigator.clipboard.writeText(c),i.value=e,setTimeout(()=>{i.value=""},2e3)}catch(f){console.error("Failed to copy:",f)}}return N(r,()=>{v(()=>y.highlightAll())}),B(()=>{v(()=>y.highlightAll())}),(c,e)=>(b(),u("div",U,[t("div",S,[e[46]||(e[46]=t("h1",{class:"text-4xl font-bold tracking-tight"},"Table",-1)),e[47]||(e[47]=t("span",null,"A responsive table component.",-1)),e[48]||(e[48]=t("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Installation",-1)),t("div",D,[t("div",E,[e[7]||(e[7]=t("span",{class:"text-xs text-zinc-600 dark:text-zinc-400 font-medium"},"bash",-1)),t("button",{onClick:e[0]||(e[0]=f=>w(h,"codeblock1")),class:"inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[i.value!=="codeblock1"?(b(),u("svg",O,[...e[5]||(e[5]=[t("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),t("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(b(),u("svg",q,[...e[6]||(e[6]=[t("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),s(" "+m(i.value==="codeblock1"?"Copied!":"Copy"),1)])]),t("pre",{class:"p-4 text-sm bg-zinc-950 text-zinc-50 overflow-x-auto"},[t("code",{class:"block font-mono !p-0 language-bash"},m(h))])]),e[49]||(e[49]=t("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Example",-1)),t("div",F,[t("div",G,[a(l(k),null,{default:o(()=>[a(l(M),null,{default:o(()=>[...e[8]||(e[8]=[s("A list of your recent invoices.",-1)])]),_:1}),a(l(A),null,{default:o(()=>[a(l(d),null,{default:o(()=>[a(l(T),{class:"w-[100px]"},{default:o(()=>[...e[9]||(e[9]=[s("Invoice",-1)])]),_:1}),a(l(T),null,{default:o(()=>[...e[10]||(e[10]=[s("Status",-1)])]),_:1}),a(l(T),null,{default:o(()=>[...e[11]||(e[11]=[s("Method",-1)])]),_:1}),a(l(T),{class:"text-right"},{default:o(()=>[...e[12]||(e[12]=[s("Amount",-1)])]),_:1})]),_:1})]),_:1}),a(l(j),null,{default:o(()=>[a(l(d),null,{default:o(()=>[a(l(n),{class:"font-medium"},{default:o(()=>[...e[13]||(e[13]=[s("INV001",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[14]||(e[14]=[s("Paid",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[15]||(e[15]=[s("Credit Card",-1)])]),_:1}),a(l(n),{class:"text-right"},{default:o(()=>[...e[16]||(e[16]=[s("$250.00",-1)])]),_:1})]),_:1}),a(l(d),null,{default:o(()=>[a(l(n),{class:"font-medium"},{default:o(()=>[...e[17]||(e[17]=[s("INV002",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[18]||(e[18]=[s("Pending",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[19]||(e[19]=[s("PayPal",-1)])]),_:1}),a(l(n),{class:"text-right"},{default:o(()=>[...e[20]||(e[20]=[s("$150.00",-1)])]),_:1})]),_:1}),a(l(d),null,{default:o(()=>[a(l(n),{class:"font-medium"},{default:o(()=>[...e[21]||(e[21]=[s("INV003",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[22]||(e[22]=[s("Unpaid",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[23]||(e[23]=[s("Bank Transfer",-1)])]),_:1}),a(l(n),{class:"text-right"},{default:o(()=>[...e[24]||(e[24]=[s("$350.00",-1)])]),_:1})]),_:1}),a(l(d),null,{default:o(()=>[a(l(n),{class:"font-medium"},{default:o(()=>[...e[25]||(e[25]=[s("INV004",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[26]||(e[26]=[s("Paid",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[27]||(e[27]=[s("Credit Card",-1)])]),_:1}),a(l(n),{class:"text-right"},{default:o(()=>[...e[28]||(e[28]=[s("$450.00",-1)])]),_:1})]),_:1}),a(l(d),null,{default:o(()=>[a(l(n),{class:"font-medium"},{default:o(()=>[...e[29]||(e[29]=[s("INV005",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[30]||(e[30]=[s("Paid",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[31]||(e[31]=[s("PayPal",-1)])]),_:1}),a(l(n),{class:"text-right"},{default:o(()=>[...e[32]||(e[32]=[s("$550.00",-1)])]),_:1})]),_:1}),a(l(d),null,{default:o(()=>[a(l(n),{class:"font-medium"},{default:o(()=>[...e[33]||(e[33]=[s("INV006",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[34]||(e[34]=[s("Pending",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[35]||(e[35]=[s("Bank Transfer",-1)])]),_:1}),a(l(n),{class:"text-right"},{default:o(()=>[...e[36]||(e[36]=[s("$200.00",-1)])]),_:1})]),_:1}),a(l(d),null,{default:o(()=>[a(l(n),{class:"font-medium"},{default:o(()=>[...e[37]||(e[37]=[s("INV007",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[38]||(e[38]=[s("Unpaid",-1)])]),_:1}),a(l(n),null,{default:o(()=>[...e[39]||(e[39]=[s("Credit Card",-1)])]),_:1}),a(l(n),{class:"text-right"},{default:o(()=>[...e[40]||(e[40]=[s("$300.00",-1)])]),_:1})]),_:1})]),_:1})]),_:1})]),t("div",Q,[t("button",{onClick:e[1]||(e[1]=f=>x.value=!x.value),class:"flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"},[e[42]||(e[42]=t("span",{class:"font-medium"},"Code",-1)),(b(),u("svg",{class:p([x.value?"rotate-180":"","transition-transform duration-200"]),xmlns:"http://www.w3.org/2000/svg",width:"16",height:"16",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[...e[41]||(e[41]=[t("path",{d:"m6 9 6 6 6-6"},null,-1)])],2))]),x.value?(b(),u("div",W,[t("div",Y,[t("div",J,[t("button",{onClick:e[2]||(e[2]=f=>r.value="auto"),class:p([r.value==="auto"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])}," Auto ",2),t("button",{onClick:e[3]||(e[3]=f=>r.value="vue"),class:p([r.value==="vue"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])}," Vue ",2)]),t("button",{onClick:e[4]||(e[4]=f=>w(r.value==="auto"?z:$,"table-basic")),class:"inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[i.value!=="table-basic"?(b(),u("svg",K,[...e[43]||(e[43]=[t("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),t("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(b(),u("svg",L,[...e[44]||(e[44]=[t("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),s(" "+m(i.value==="table-basic"?"Copied!":"Copy"),1)])]),t("pre",X,[t("code",{class:p("block font-mono !p-0 language-"+(r.value==="auto"?"auto":"html"))},m(r.value==="auto"?z:$),3)])])):R("",!0)])]),e[50]||(e[50]=t("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Data Table",-1)),e[51]||(e[51]=t("span",{class:"text-muted-foreground"},"You can use the Table component to build more complex data tables. Combine it with @tanstack/vue-table to create tables with sorting, filtering and pagination.",-1)),e[52]||(e[52]=t("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Properties",-1)),a(l(k),null,{default:o(()=>[...e[45]||(e[45]=[t("thead",{class:"bg-muted/50"},[t("tr",null,[t("th",{class:"border px-4 py-2 text-left font-semibold"},"Component"),t("th",{class:"border px-4 py-2 text-left font-semibold"},"Description")])],-1),t("tbody",null,[t("tr",null,[t("td",{class:"border px-4 py-2"},"Table"),t("td",{class:"border px-4 py-2"},"Main table container")]),t("tr",null,[t("td",{class:"border px-4 py-2"},"TableHeader"),t("td",{class:"border px-4 py-2"},"Header section")]),t("tr",null,[t("td",{class:"border px-4 py-2"},"TableBody"),t("td",{class:"border px-4 py-2"},"Body section")]),t("tr",null,[t("td",{class:"border px-4 py-2"},"TableCaption"),t("td",{class:"border px-4 py-2"},"Table caption/description")]),t("tr",null,[t("td",{class:"border px-4 py-2"},"TableRow"),t("td",{class:"border px-4 py-2"},"Table row")]),t("tr",null,[t("td",{class:"border px-4 py-2"},"TableHead"),t("td",{class:"border px-4 py-2"},"Header cell")]),t("tr",null,[t("td",{class:"border px-4 py-2"},"TableCell"),t("td",{class:"border px-4 py-2"},"Data cell")])],-1)])]),_:1})])]))}}),le=H(Z,[["__scopeId","data-v-bb88c33d"]]);export{le as default};
