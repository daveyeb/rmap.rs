function _1(e){return e`<div style="color: grey; font: 13px/25.5px var(--sans-serif); text-transform: uppercase;"><h1 style="display: none;">Color legend</h1><a href="https://d3js.org/">D3</a> › <a href="/@d3/gallery">Gallery</a></div>

# Color legend

A simple legend for a [color scale](/@d3/color-schemes). Supports [continuous](/@d3/continuous-scales), [sequential](/@d3/sequential-scales), [diverging](/@d3/diverging-scales), [quantize, quantile, threshold](/@d3/quantile-quantize-and-threshold-scales) and [ordinal](/@d3/d3-scaleordinal) scales. To use:

~~~js
import {Legend, Swatches} from "@d3/color-legend"
~~~

Then call the legend function as shown below. (For ordinal scales, also consider the swatches function.)`}function _2(e,t){return e(t.scaleSequential([0,100],t.interpolateViridis),{title:"Temperature (°F)"})}function _3(e,t){return e(t.scaleSequentialSqrt([0,1],t.interpolateTurbo),{title:"Speed (kts)"})}function _4(e,t){return e(t.scaleDiverging([-.1,0,.1],t.interpolatePiYG),{title:"Daily change",tickFormat:"+%"})}function _5(e,t){return e(t.scaleDivergingSqrt([-.1,0,.1],t.interpolateRdBu),{title:"Daily change",tickFormat:"+%"})}function _6(e,t){return e(t.scaleSequentialLog([1,100],t.interpolateBlues),{title:"Energy (joules)",ticks:10})}function _7(e,t){return e(t.scaleSequentialQuantile(t.range(100).map((()=>Math.random()**2)),t.interpolateBlues),{title:"Quantile",tickFormat:".2f"})}function _8(e,t){return e(t.scaleSqrt([-100,0,100],["blue","white","red"]),{title:"Temperature (°C)"})}function _9(e,t){return e(t.scaleQuantize([1,10],t.schemePurples[9]),{title:"Unemployment rate (%)"})}function _10(e,t){return e(t.scaleQuantile(t.range(1e3).map(t.randomNormal(100,20)),t.schemeSpectral[9]),{title:"Height (cm)",tickFormat:".0f"})}function _11(e,t){return e(t.scaleThreshold([2.5,3.1,3.5,3.9,6,7,8,9.5],t.schemeRdBu[9]),{title:"Unemployment rate (%)",tickSize:0})}function _12(e,t){return e(t.scaleOrdinal(["<10","10-19","20-29","30-39","40-49","50-59","60-69","70-79","≥80"],t.schemeSpectral[10]),{title:"Age (years)",tickSize:0})}function _13(e){return e`But wait, there’s more!

How about swatches for ordinal color scales? Both variable-width swatches and [column layout](https://developer.mozilla.org/en-US/docs/Web/CSS/columns) are supported.`}function _14(e,t){return e(t.scaleOrdinal(["blueberries","oranges","apples"],t.schemeCategory10))}function _15(e,t){return e(t.scaleOrdinal(["Wholesale and Retail Trade","Manufacturing","Leisure and hospitality","Business services","Construction","Education and Health","Government","Finance","Self-employed","Other"],t.schemeTableau10),{columns:"180px"})}function _16(e){return e`---

## Implementation`}function _Legend(e){return function(t,{title:n,tickSize:a=6,width:i=320,height:r=44+a,marginTop:l=18,marginRight:o=0,marginBottom:s=16+a,marginLeft:d=0,ticks:c=i/64,tickFormat:u,tickValues:h}={}){function g(e,t=256){const n=document.createElement("canvas");n.width=t,n.height=1;const a=n.getContext("2d");for(let n=0;n<t;++n)a.fillStyle=e(n/(t-1)),a.fillRect(n,0,1,1);return n}const f=e.create("svg").attr("width",i).attr("height",r).attr("viewBox",[0,0,i,r]).style("overflow","visible").style("display","block");let m,p=e=>e.selectAll(".tick line").attr("y1",l+s-r);if(t.interpolate){const n=Math.min(t.domain().length,t.range().length);m=t.copy().rangeRound(e.quantize(e.interpolate(d,i-o),n)),f.append("image").attr("x",d).attr("y",l).attr("width",i-d-o).attr("height",r-l-s).attr("preserveAspectRatio","none").attr("xlink:href",g(t.copy().domain(e.quantize(e.interpolate(0,1),n))).toDataURL())}else if(t.interpolator){if(m=Object.assign(t.copy().interpolator(e.interpolateRound(d,i-o)),{range:()=>[d,i-o]}),f.append("image").attr("x",d).attr("y",l).attr("width",i-d-o).attr("height",r-l-s).attr("preserveAspectRatio","none").attr("xlink:href",g(t.interpolator()).toDataURL()),!m.ticks){if(void 0===h){const n=Math.round(c+1);h=e.range(n).map((a=>e.quantile(t.domain(),a/(n-1))))}"function"!=typeof u&&(u=e.format(void 0===u?",f":u))}}else if(t.invertExtent){const n=t.thresholds?t.thresholds():t.quantiles?t.quantiles():t.domain(),a=void 0===u?e=>e:"string"==typeof u?e.format(u):u;m=e.scaleLinear().domain([-1,t.range().length-1]).rangeRound([d,i-o]),f.append("g").selectAll("rect").data(t.range()).join("rect").attr("x",((e,t)=>m(t-1))).attr("y",l).attr("width",((e,t)=>m(t)-m(t-1))).attr("height",r-l-s).attr("fill",(e=>e)),h=e.range(n.length),u=e=>a(n[e],e)}else m=e.scaleBand().domain(t.domain()).rangeRound([d,i-o]),f.append("g").selectAll("rect").data(t.domain()).join("rect").attr("x",m).attr("y",l).attr("width",Math.max(0,m.bandwidth()-1)).attr("height",r-l-s).attr("fill",t),p=()=>{};return f.append("g").attr("transform",`translate(0,${r-s})`).call(e.axisBottom(m).ticks(c,"string"==typeof u?u:void 0).tickFormat("function"==typeof u?u:void 0).tickSize(a).tickValues(h)).call(p).call((e=>e.select(".domain").remove())).call((e=>e.append("text").attr("x",d).attr("y",l+s-r-6).attr("fill","currentColor").attr("text-anchor","start").attr("font-weight","bold").attr("class","title").text(n))),f.node()}}function _legend(e){return function({color:t,...n}){return e(t,n)}}function _Swatches(e,t){return function(n,{columns:a=null,format:i,unknown:r,swatchSize:l=15,swatchWidth:o=l,swatchHeight:s=l,marginLeft:d=0}={}){const c=`-swatches-${Math.random().toString(16).slice(2)}`,u=null==r?void 0:n.unknown(),h=null==u||u===e.scaleImplicit?[]:[u],g=n.domain().concat(h);return void 0===i&&(i=e=>e===u?r:e),null!==a?t.html`<div style="display: flex; align-items: center; margin-left: ${+d}px; min-height: 33px; font: 10px sans-serif;">
  <style>

.${c}-item {
  break-inside: avoid;
  display: flex;
  align-items: center;
  padding-bottom: 1px;
}

.${c}-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: calc(100% - ${+o}px - 0.5em);
}

.${c}-swatch {
  width: ${+o}px;
  height: ${+s}px;
  margin: 0 0.5em 0 0;
}

  </style>
  <div style=${{width:"100%",columns:a}}>${g.map((e=>{const a=`${i(e)}`;return t.html`<div class=${c}-item>
      <div class=${c}-swatch style=${{background:n(e)}}></div>
      <div class=${c}-label title=${a}>${a}</div>
    </div>`}))}
  </div>
</div>`:t.html`<div style="display: flex; align-items: center; min-height: 33px; margin-left: ${+d}px; font: 10px sans-serif;">
  <style>

.${c} {
  display: inline-flex;
  align-items: center;
  margin-right: 1em;
}

.${c}::before {
  content: "";
  width: ${+o}px;
  height: ${+s}px;
  margin-right: 0.5em;
  background: var(--color);
}

  </style>
  <div>${g.map((e=>t.html`<span class="${c}" style="--color: ${n(e)}">${i(e)}</span>`))}</div>`}}function _swatches(e){return function({color:t,...n}){return e(t,n)}}export default function define(e,t){const n=e.module();return n.variable(t()).define(["md"],_1),n.variable(t()).define(["Legend","d3"],_2),n.variable(t()).define(["Legend","d3"],_3),n.variable(t()).define(["Legend","d3"],_4),n.variable(t()).define(["Legend","d3"],_5),n.variable(t()).define(["Legend","d3"],_6),n.variable(t()).define(["Legend","d3"],_7),n.variable(t()).define(["Legend","d3"],_8),n.variable(t()).define(["Legend","d3"],_9),n.variable(t()).define(["Legend","d3"],_10),n.variable(t()).define(["Legend","d3"],_11),n.variable(t()).define(["Legend","d3"],_12),n.variable(t()).define(["md"],_13),n.variable(t()).define(["Swatches","d3"],_14),n.variable(t()).define(["Swatches","d3"],_15),n.variable(t()).define(["md"],_16),n.variable(t("Legend")).define("Legend",["d3"],_Legend),n.variable(t("legend")).define("legend",["Legend"],_legend),n.variable(t("Swatches")).define("Swatches",["d3","htl"],_Swatches),n.variable(t("swatches")).define("swatches",["Swatches"],_swatches),n}