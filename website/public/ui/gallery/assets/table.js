import{d as h,o as t,a as T,G as y,u as a,H as P,q as V,K as B,O as H,Q as g,R as v,b as l,g as n,U as C,e as o,w as s,V as I,r as f}from"./index.js";import{_ as c}from"./Table.vue_vue_type_script_setup_true_lang.js";import{_ as N,a as i,b as k,c as A,d as r}from"./TableHeader.vue_vue_type_script_setup_true_lang.js";const j=h({__name:"TableCaption",props:{class:{type:[Boolean,null,String,Object,Array]}},setup(p){const d=p;return(u,b)=>(t(),T("caption",{class:y(a(P)("mt-4 text-sm text-muted-foreground",d.class))},[V(u.$slots,"default")],2))}}),M={class:"flex flex-col"},U={class:"relative rounded-lg border overflow-hidden"},S={class:"flex items-center justify-between px-4 py-3 bg-zinc-100 dark:bg-zinc-800 border-b"},D={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},E={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},O={class:"rounded-lg border overflow-hidden"},q={class:"flex items-center justify-center p-4 min-h-[100px] bg-zinc-100 dark:bg-zinc-900"},F={class:"border-t"},G={key:0,class:"border-t"},K={class:"flex items-center justify-between bg-zinc-100 dark:bg-zinc-800"},Q={class:"flex"},Y={key:0,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},J={key:1,xmlns:"http://www.w3.org/2000/svg",width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},L={class:"overflow-x-auto p-4 text-sm bg-zinc-950 text-zinc-50"},z=`table {
    table-caption (text: "A list of your recent invoices.") {}
    table-header {
        table-row {
            table-head (style: "w-[100px]", text: "Invoice") {}
            table-head (text: "Status") {}
            table-head (text: "Method") {}
            table-head (text: "Amount", style: "text-right") {}
        }
    }
    table-body {
        table-row {
            table-cell (style: "font-medium", text: "INV001") {}
            table-cell (text: "Paid") {}
            table-cell (text: "Credit Card") {}
            table-cell (style: "text-right", text: "$250.00") {}
        }
        table-row {
            table-cell (text: "INV002", style: "font-medium") {}
            table-cell (text: "Pending") {}
            table-cell (text: "PayPal") {}
            table-cell (style: "text-right", text: "$150.00") {}
        }
        table-row {
            table-cell (text: "INV003", style: "font-medium") {}
            table-cell (text: "Unpaid") {}
            table-cell (text: "Bank Transfer") {}
            table-cell (text: "$350.00", style: "text-right") {}
        }
        table-row {
            table-cell (text: "INV004", style: "font-medium") {}
            table-cell (text: "Paid") {}
            table-cell (text: "Credit Card") {}
            table-cell (text: "$450.00", style: "text-right") {}
        }
        table-row {
            table-cell (text: "INV005", style: "font-medium") {}
            table-cell (text: "Paid") {}
            table-cell (text: "PayPal") {}
            table-cell (style: "text-right", text: "$550.00") {}
        }
        table-row {
            table-cell (text: "INV006", style: "font-medium") {}
            table-cell (text: "Pending") {}
            table-cell (text: "Bank Transfer") {}
            table-cell (style: "text-right", text: "$200.00") {}
        }
        table-row {
            table-cell (style: "font-medium", text: "INV007") {}
            table-cell (text: "Unpaid") {}
            table-cell (text: "Credit Card") {}
            table-cell (style: "text-right", text: "$300.00") {}
        }
    }
}
`,R=`<Table :key="'Table-1'">
  <TableCaption :key="'TableCaption-2'">A list of your recent invoices.</TableCaption>
  <TableHeader :key="'TableHeader-3'">
    <TableRow :key="'TableRow-4'">
      <TableHead class="w-[100px]" :key="'TableHead-5'">Invoice</TableHead>
      <TableHead :key="'TableHead-6'">Status</TableHead>
      <TableHead :key="'TableHead-7'">Method</TableHead>
      <TableHead class="text-right" :key="'TableHead-8'">Amount</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody :key="'TableBody-9'">
    <TableRow :key="'TableRow-10'">
      <TableCell class="font-medium" :key="'TableCell-11'">INV001</TableCell>
      <TableCell :key="'TableCell-12'">Paid</TableCell>
      <TableCell :key="'TableCell-13'">Credit Card</TableCell>
      <TableCell class="text-right" :key="'TableCell-14'">$250.00</TableCell>
    </TableRow>
    <TableRow :key="'TableRow-15'">
      <TableCell class="font-medium" :key="'TableCell-16'">INV002</TableCell>
      <TableCell :key="'TableCell-17'">Pending</TableCell>
      <TableCell :key="'TableCell-18'">PayPal</TableCell>
      <TableCell class="text-right" :key="'TableCell-19'">$150.00</TableCell>
    </TableRow>
    <TableRow :key="'TableRow-20'">
      <TableCell class="font-medium" :key="'TableCell-21'">INV003</TableCell>
      <TableCell :key="'TableCell-22'">Unpaid</TableCell>
      <TableCell :key="'TableCell-23'">Bank Transfer</TableCell>
      <TableCell class="text-right" :key="'TableCell-24'">$350.00</TableCell>
    </TableRow>
    <TableRow :key="'TableRow-25'">
      <TableCell class="font-medium" :key="'TableCell-26'">INV004</TableCell>
      <TableCell :key="'TableCell-27'">Paid</TableCell>
      <TableCell :key="'TableCell-28'">Credit Card</TableCell>
      <TableCell class="text-right" :key="'TableCell-29'">$450.00</TableCell>
    </TableRow>
    <TableRow :key="'TableRow-30'">
      <TableCell class="font-medium" :key="'TableCell-31'">INV005</TableCell>
      <TableCell :key="'TableCell-32'">Paid</TableCell>
      <TableCell :key="'TableCell-33'">PayPal</TableCell>
      <TableCell class="text-right" :key="'TableCell-34'">$550.00</TableCell>
    </TableRow>
    <TableRow :key="'TableRow-35'">
      <TableCell class="font-medium" :key="'TableCell-36'">INV006</TableCell>
      <TableCell :key="'TableCell-37'">Pending</TableCell>
      <TableCell :key="'TableCell-38'">Bank Transfer</TableCell>
      <TableCell class="text-right" :key="'TableCell-39'">$200.00</TableCell>
    </TableRow>
    <TableRow :key="'TableRow-40'">
      <TableCell class="font-medium" :key="'TableCell-41'">INV007</TableCell>
      <TableCell :key="'TableCell-42'">Unpaid</TableCell>
      <TableCell :key="'TableCell-43'">Credit Card</TableCell>
      <TableCell class="text-right" :key="'TableCell-44'">$300.00</TableCell>
    </TableRow>
  </TableBody>
</Table>
`,$="npx shadcn-vue@latest add table",_=h({__name:"table",setup(p){const d=f(""),u=f(!0),b=f("auto");async function m(w,e){try{await navigator.clipboard.writeText(w),d.value=e,setTimeout(()=>{d.value=""},2e3)}catch(x){console.error("Failed to copy:",x)}}return B(b,()=>{g(()=>v.highlightAll())}),H(()=>{g(()=>v.highlightAll())}),(w,e)=>(t(),T("div",M,[e[46]||(e[46]=l("h1",null,"Table",-1)),e[47]||(e[47]=l("span",null,"A responsive table component.",-1)),e[48]||(e[48]=l("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Installation",-1)),l("div",U,[l("div",S,[e[7]||(e[7]=l("span",{class:"text-xs text-zinc-600 dark:text-zinc-400 font-medium"},"bash",-1)),l("button",{onClick:e[0]||(e[0]=x=>m($,"codeblock1")),class:"inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[d.value!=="codeblock1"?(t(),T("svg",D,[...e[5]||(e[5]=[l("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),l("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(t(),T("svg",E,[...e[6]||(e[6]=[l("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),n(" "+C(d.value==="codeblock1"?"Copied!":"Copy"),1)])]),l("pre",{class:"p-4 text-sm bg-zinc-950 text-zinc-50 overflow-x-auto"},[l("code",{class:"block font-mono !p-0 language-bash"},C($))])]),e[49]||(e[49]=l("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Example",-1)),l("div",O,[l("div",q,[(t(),o(a(c),{key:"Table-45"},{default:s(()=>[(t(),o(a(j),{key:"TableCaption-46"},{default:s(()=>[...e[8]||(e[8]=[n("A list of your recent invoices.",-1)])]),_:1})),(t(),o(a(N),{key:"TableHeader-47"},{default:s(()=>[(t(),o(a(i),{key:"TableRow-48"},{default:s(()=>[(t(),o(a(k),{class:"w-[100px]",key:"TableHead-49"},{default:s(()=>[...e[9]||(e[9]=[n("Invoice",-1)])]),_:1})),(t(),o(a(k),{key:"TableHead-50"},{default:s(()=>[...e[10]||(e[10]=[n("Status",-1)])]),_:1})),(t(),o(a(k),{key:"TableHead-51"},{default:s(()=>[...e[11]||(e[11]=[n("Method",-1)])]),_:1})),(t(),o(a(k),{class:"text-right",key:"TableHead-52"},{default:s(()=>[...e[12]||(e[12]=[n("Amount",-1)])]),_:1}))]),_:1}))]),_:1})),(t(),o(a(A),{key:"TableBody-53"},{default:s(()=>[(t(),o(a(i),{key:"TableRow-54"},{default:s(()=>[(t(),o(a(r),{class:"font-medium",key:"TableCell-55"},{default:s(()=>[...e[13]||(e[13]=[n("INV001",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-56"},{default:s(()=>[...e[14]||(e[14]=[n("Paid",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-57"},{default:s(()=>[...e[15]||(e[15]=[n("Credit Card",-1)])]),_:1})),(t(),o(a(r),{class:"text-right",key:"TableCell-58"},{default:s(()=>[...e[16]||(e[16]=[n("$250.00",-1)])]),_:1}))]),_:1})),(t(),o(a(i),{key:"TableRow-59"},{default:s(()=>[(t(),o(a(r),{class:"font-medium",key:"TableCell-60"},{default:s(()=>[...e[17]||(e[17]=[n("INV002",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-61"},{default:s(()=>[...e[18]||(e[18]=[n("Pending",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-62"},{default:s(()=>[...e[19]||(e[19]=[n("PayPal",-1)])]),_:1})),(t(),o(a(r),{class:"text-right",key:"TableCell-63"},{default:s(()=>[...e[20]||(e[20]=[n("$150.00",-1)])]),_:1}))]),_:1})),(t(),o(a(i),{key:"TableRow-64"},{default:s(()=>[(t(),o(a(r),{class:"font-medium",key:"TableCell-65"},{default:s(()=>[...e[21]||(e[21]=[n("INV003",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-66"},{default:s(()=>[...e[22]||(e[22]=[n("Unpaid",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-67"},{default:s(()=>[...e[23]||(e[23]=[n("Bank Transfer",-1)])]),_:1})),(t(),o(a(r),{class:"text-right",key:"TableCell-68"},{default:s(()=>[...e[24]||(e[24]=[n("$350.00",-1)])]),_:1}))]),_:1})),(t(),o(a(i),{key:"TableRow-69"},{default:s(()=>[(t(),o(a(r),{class:"font-medium",key:"TableCell-70"},{default:s(()=>[...e[25]||(e[25]=[n("INV004",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-71"},{default:s(()=>[...e[26]||(e[26]=[n("Paid",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-72"},{default:s(()=>[...e[27]||(e[27]=[n("Credit Card",-1)])]),_:1})),(t(),o(a(r),{class:"text-right",key:"TableCell-73"},{default:s(()=>[...e[28]||(e[28]=[n("$450.00",-1)])]),_:1}))]),_:1})),(t(),o(a(i),{key:"TableRow-74"},{default:s(()=>[(t(),o(a(r),{class:"font-medium",key:"TableCell-75"},{default:s(()=>[...e[29]||(e[29]=[n("INV005",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-76"},{default:s(()=>[...e[30]||(e[30]=[n("Paid",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-77"},{default:s(()=>[...e[31]||(e[31]=[n("PayPal",-1)])]),_:1})),(t(),o(a(r),{class:"text-right",key:"TableCell-78"},{default:s(()=>[...e[32]||(e[32]=[n("$550.00",-1)])]),_:1}))]),_:1})),(t(),o(a(i),{key:"TableRow-79"},{default:s(()=>[(t(),o(a(r),{class:"font-medium",key:"TableCell-80"},{default:s(()=>[...e[33]||(e[33]=[n("INV006",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-81"},{default:s(()=>[...e[34]||(e[34]=[n("Pending",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-82"},{default:s(()=>[...e[35]||(e[35]=[n("Bank Transfer",-1)])]),_:1})),(t(),o(a(r),{class:"text-right",key:"TableCell-83"},{default:s(()=>[...e[36]||(e[36]=[n("$200.00",-1)])]),_:1}))]),_:1})),(t(),o(a(i),{key:"TableRow-84"},{default:s(()=>[(t(),o(a(r),{class:"font-medium",key:"TableCell-85"},{default:s(()=>[...e[37]||(e[37]=[n("INV007",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-86"},{default:s(()=>[...e[38]||(e[38]=[n("Unpaid",-1)])]),_:1})),(t(),o(a(r),{key:"TableCell-87"},{default:s(()=>[...e[39]||(e[39]=[n("Credit Card",-1)])]),_:1})),(t(),o(a(r),{class:"text-right",key:"TableCell-88"},{default:s(()=>[...e[40]||(e[40]=[n("$300.00",-1)])]),_:1}))]),_:1}))]),_:1}))]),_:1}))]),l("div",F,[l("button",{onClick:e[1]||(e[1]=x=>u.value=!u.value),class:"flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"},[e[42]||(e[42]=l("span",{class:"font-medium"},"Code",-1)),(t(),T("svg",{class:y([u.value?"rotate-180":"","transition-transform duration-200"]),xmlns:"http://www.w3.org/2000/svg",width:"16",height:"16",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":"2","stroke-linecap":"round","stroke-linejoin":"round"},[...e[41]||(e[41]=[l("path",{d:"m6 9 6 6 6-6"},null,-1)])],2))]),u.value?(t(),T("div",G,[l("div",K,[l("div",Q,[l("button",{onClick:e[2]||(e[2]=x=>b.value="auto"),class:y([b.value==="auto"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])}," Auto ",2),l("button",{onClick:e[3]||(e[3]=x=>b.value="vue"),class:y([b.value==="vue"?"bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100 border-b-2 border-primary -mb-px":"text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 border-b-2 border-transparent","px-4 py-2 text-xs font-medium transition-colors"])}," Vue ",2)]),l("button",{onClick:e[4]||(e[4]=x=>m(b.value==="auto"?z:R,"table-basic")),class:"inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-900 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"},[d.value!=="table-basic"?(t(),T("svg",Y,[...e[43]||(e[43]=[l("rect",{width:"14",height:"14",x:"8",y:"8",rx:"2",ry:"2"},null,-1),l("path",{d:"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"},null,-1)])])):(t(),T("svg",J,[...e[44]||(e[44]=[l("path",{d:"M20 6 9 17l-5-5"},null,-1)])])),n(" "+C(d.value==="table-basic"?"Copied!":"Copy"),1)])]),l("pre",L,[l("code",{class:y("block font-mono !p-0 language-"+(b.value==="auto"?"auto":"html"))},C(b.value==="auto"?z:R),3)])])):I("",!0)])]),e[50]||(e[50]=l("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Data Table",-1)),e[51]||(e[51]=l("span",{class:"text-muted-foreground"},"You can use the Table component to build more complex data tables. Combine it with @tanstack/vue-table to create tables with sorting, filtering and pagination.",-1)),e[52]||(e[52]=l("h2",{class:"text-2xl font-semibold tracking-tight mt-8"},"Properties",-1)),(t(),o(a(c),{key:"Table-89"},{default:s(()=>[...e[45]||(e[45]=[l("thead",{class:"bg-muted/50"},[l("tr",null,[l("th",{class:"border px-4 py-2 text-left font-semibold"},"Component"),l("th",{class:"border px-4 py-2 text-left font-semibold"},"Description")])],-1),l("tbody",null,[l("tr",null,[l("td",{class:"border px-4 py-2"},"Table"),l("td",{class:"border px-4 py-2"},"Main table container")]),l("tr",null,[l("td",{class:"border px-4 py-2"},"TableHeader"),l("td",{class:"border px-4 py-2"},"Header section")]),l("tr",null,[l("td",{class:"border px-4 py-2"},"TableBody"),l("td",{class:"border px-4 py-2"},"Body section")]),l("tr",null,[l("td",{class:"border px-4 py-2"},"TableCaption"),l("td",{class:"border px-4 py-2"},"Table caption/description")]),l("tr",null,[l("td",{class:"border px-4 py-2"},"TableRow"),l("td",{class:"border px-4 py-2"},"Table row")]),l("tr",null,[l("td",{class:"border px-4 py-2"},"TableHead"),l("td",{class:"border px-4 py-2"},"Header cell")]),l("tr",null,[l("td",{class:"border px-4 py-2"},"TableCell"),l("td",{class:"border px-4 py-2"},"Data cell")])],-1)])]),_:1}))]))}});export{_ as default};
