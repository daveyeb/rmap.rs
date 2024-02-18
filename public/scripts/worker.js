import * as Comlink from "./comlink.mjs";
// import * as FeatureDectect from 'https://unpkg.com/wasm-feature-detect@1.5.1/dist/esm/index.js';


// Wrap wasm-bindgen exports (the `generate` function) to add time measurement.
function wrapExports({ parse_blobs }) {
    return ({ blobs }) => {
        try {
            const start = performance.now();
            const rawImageData = parse_blobs(blobs);
            const time = performance.now() - start;
            return {
                // Little perf boost to transfer data to the main thread w/o copying.
                rawImageData: rawImageData,
                time
            };
        } catch (error) {
            console.log("trying to catch from worker", error)
        }

    };
}

async function initHandlers() {
    let [multi] = await Promise.all([
        (async () => {
            // If threads are unsupported in this browser, skip this handler.
            // if (!(await FeatureDectect.threads())) return;
            const multiThread = await import(
                '/pkg/rmap.js'
            );

            await multiThread.default();
            await multiThread.initThreadPool(navigator.hardwareConcurrency);
            return wrapExports(multiThread);
        })()
    ]);

    return Comlink.proxy({
        supportsThreads: !!multi,
        multi
    });
}

Comlink.expose({
    handlers: initHandlers()
});