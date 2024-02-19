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

 import{e,i as t}from"./directive-e2d48b9c.js";import"./settings-daf72103.js";
 /**
  * @license
  *
  * Copyright IBM Corp. 2019, 2023
  *
  * This source code is licensed under the Apache-2.0 license found in the
  * LICENSE file in the root directory of this source tree.
  */const s=new WeakMap;const r=e(class extends t{update(e,[t]){const{element:r}=e,n=s.get(e);return n&&Object.keys(n).forEach((e=>{e in t||r.removeAttribute(e)})),Object.keys(t).forEach((e=>{const s=t[e];n&&Object.is(s,n[e])||void 0===s||r.setAttribute(e,s)})),s.set(e,t),this.render(t)}render(e){return e}});export{r as s};