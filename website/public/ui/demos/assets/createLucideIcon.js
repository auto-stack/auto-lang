import{aX as a}from"./index.js";/**
 * @license lucide-vue-next v0.312.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */var e={xmlns:"http://www.w3.org/2000/svg",width:24,height:24,viewBox:"0 0 24 24",fill:"none",stroke:"currentColor","stroke-width":2,"stroke-linecap":"round","stroke-linejoin":"round"};/**
 * @license lucide-vue-next v0.312.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */const h=t=>t.replace(/([a-z0-9])([A-Z])/g,"$1-$2").toLowerCase(),k=(t,i)=>({size:r,strokeWidth:o=2,absoluteStrokeWidth:c,color:l,class:w,...n},{attrs:u,slots:s})=>a("svg",{...e,width:r||e.width,height:r||e.height,stroke:l||e.stroke,"stroke-width":c?Number(o)*24/Number(r):o,...u,class:["lucide",`lucide-${h(t)}`],...n},[...i.map(d=>a(...d)),...s.default?[s.default()]:[]]);export{k as c};
