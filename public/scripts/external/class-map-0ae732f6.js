/**
 * @license
 *
 * Copyright IBM Corp. 2019, 2020
 *
 * This source code is licensed under the Apache-2.0 license found in the
 * LICENSE file in the root directory of this source tree.
 */
/**
 * @license
 * 
 * This bundle contains the following third-party dependencies:
 * 
 * flatpickr:
 * 
 flatpickr v4.6.1, @license MIT
 * 
 * lit-element:
 * 
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 * 
 * lit-html:
 * 
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 * 
 * @lit/reactive-element:
 * 
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 * 
 * @carbon/layout:
 * 
 * Copyright IBM Corp. 2018, 2023
 *
 * This source code is licensed under the Apache-2.0 license found in the
 * LICENSE file in the root directory of this source tree.
 * 
 * Also refer to the following links for the license of other third-party dependencies:
 * 
 * https://www.npmjs.com/package/@carbon/icons
 * https://www.npmjs.com/package/lit
 * https://www.npmjs.com/package/lodash-es
 */

 import{T as t}from"./settings-daf72103.js";import{e as i,i as s,t as e}from"./directive-e2d48b9c.js";
 /**
  * @license
  * Copyright 2018 Google LLC
  * SPDX-License-Identifier: BSD-3-Clause
  */const n=i(class extends s{constructor(t){var i;if(super(t),t.type!==e.ATTRIBUTE||"class"!==t.name||(null===(i=t.strings)||void 0===i?void 0:i.length)>2)throw Error("`classMap()` can only be used in the `class` attribute and must be the only part in the attribute.")}render(t){return" "+Object.keys(t).filter((i=>t[i])).join(" ")+" "}update(i,[s]){var e,n;if(void 0===this.it){this.it=new Set,void 0!==i.strings&&(this.nt=new Set(i.strings.join(" ").split(/\s/).filter((t=>""!==t))));for(const t in s)s[t]&&!(null===(e=this.nt)||void 0===e?void 0:e.has(t))&&this.it.add(t);return this.render(s)}const r=i.element.classList;this.it.forEach((t=>{t in s||(r.remove(t),this.it.delete(t))}));for(const t in s){const i=!!s[t];i===this.it.has(t)||(null===(n=this.nt)||void 0===n?void 0:n.has(t))||(i?(r.add(t),this.it.add(t)):(r.remove(t),this.it.delete(t)))}return t}});export{n as o};