import * as Comlink from "./comlink.mjs";
import { Runtime, Library, Inspector } from "./external/runtime.js";
import { define as tdefine, link as tlink } from "./external/treemap.js";
import { define as ndefine, link as nlink } from "./external/c019e733534da627@316.js";
import define from "./external/line_chart.helper.js"

var queue = [];
const workers = [];
var isFetching = false;
var gstats;

htmx.onLoad(function (content) {
    console.log("htmx overload")

    document.addEventListener('htmx:afterRequest', function (evt) {
        if (evt.detail.target.id == 'search-results' && evt.detail.xhr.status == 200) {
            $('#search-results').show();
            $('#dash-container').hide();
        }
    });


    function initGenButtons() {
        var genButtons = content.querySelectorAll("#gen-btn")
        var genDashButtons = content.querySelectorAll("#gen-dash-btn")
        var loadButtons = content.querySelectorAll("#load-btn")
        var retryButtons = content.querySelectorAll("#retry-btn")

        genButtons.forEach((button, i) => {
            button.addEventListener("click", async function () {
                var name = $(this).parent().parent().children(":first").text();
                var tree = await fetchTree(name, 'GET');

                var repo = { name, tree }
                queue.push(repo)

                await renderScans(repo)
            })
        })

        genDashButtons.forEach((button, i) => {
            button.addEventListener("click", async function () {
                var name = $(this).parent().parent().children(":first").text();
                var tree = await fetchTree(name, 'GET');

                var repo = { name, tree }
                queue.push(repo)

                await renderScans(repo)
            })
        })

        loadButtons.forEach((button, i) => {
            button.addEventListener("click", async function () {
                var name = $(this).parent().parent().children(":first").text();

                var links = await fetchLink(name);
                clearCharts()
                loadLinks(links)

            })
        })

        retryButtons.forEach((button, i) => {
            button.addEventListener("click", async function () {
                var name = $(this).parent().parent().children(":first").text();

                var links = await fetchLink(name);
                clearCharts()
                loadLinks(links)

            })
        })


    }

    var targetNode = document.getElementById('dash-container');
    var observer = new MutationObserver(function () {
        if (targetNode.style.display != 'none') {
            clearStats();   
            renderStats(gstats);
        }
    });
    observer.observe(targetNode, { attributes: true, childList: true });

    if (window.location.href.indexOf("dashboard") > -1) {
        fetchStats().then(stats => {
            gstats = stats
            clearStats()
            renderStats(stats);
        })
    }



    initGenButtons();
    setInterval(async () => {

        if (workers.length !== 0 || isFetching) return;

        if (queue.length >= 1) {
            var repo = queue.shift()
            await generate({ repo, blobs: await fetchBlobs(repo, isFetching) })
            // clearStats()
            // renderStats(await fetchStats())
        }

    }, 100)

});

function setUpWorker() {
    if (workers.length > 1) {
        console.log("INFO: Max workers limit reached ", worker.length)
        return;
    }

    let worker = workers.pop();
    if (worker === undefined) console.log("INFO: Creating a new worker.")
    else {
        console.log("INFO: Terminating existing worker and recreating a new one.")
        worker.terminate()
        worker = undefined
    }

    worker = new Worker(new URL(`./worker.js`, import.meta.url), {
        type: 'module'
    });
    workers.push(worker)
};


async function fetchTree(name, method) {
    const response = await fetch(new URL(`repos?full_name=${name}`, window.location.href).href, {
        method: method
    })
    return await response.json()
}

function clearCharts() {
    const networks = document.getElementById("networks");
    const treemap = document.getElementById("treemap-container");

    networks.innerHTML = '';
    treemap.innerHTML = '';
}

function clearStats() {
    const stats = document.getElementById("stats");

    stats.innerText = '';
}


async function fetchLink(name) {
    let formData = new FormData();
    formData.append('action', 1);
    formData.append('repo_name', name);

    const data = new URLSearchParams(formData)

    var response = await fetch(new URL(`/dashboard`, window.location.href).href, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: data
    })

    return response.json()
}


async function fetchStats() {
    let formData = new FormData();
    formData.append('action', 5);

    const data = new URLSearchParams(formData)

    var response = await fetch(new URL(`/dashboard`, window.location.href).href, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: data
    })

    return response.json()
}

async function putStat({ name, task, value }) {
    let formData = new FormData();
    formData.append('action', 4);
    formData.append('task', task);
    formData.append('value', value);
    formData.append('repo_name', name);

    const data = new URLSearchParams(formData)

    var response = await fetch(new URL(`/dashboard`, window.location.href).href, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: data
    })

    return response.json()
};

async function fetchBlobs({ name, tree }, isFetching) {
    isFetching = true;

    var blobs = tree.tree.tree.filter(blob => blob.type == "blob");

    var i = 0;
    var contents = [];
    var currentProgress = name.replace(/\W/g, '');

    $("." + currentProgress).addClass('currentProgress')
    for (var blob of blobs) {
        $("." + currentProgress).attr("value", (++i * 100) / blobs.length);
        $("." + currentProgress).attr("helper-text", "Fetching files");

        let chunk = await fetchBlob(blob.url);

        if (chunk.localeCompare("") != 0) i++;

        contents.push(Object.assign(blob, { "chunk": chunk }))
    }

    await putStat({ name: name, task: "fetched (files)", value: i })
    isFetching = false;
    return { blobs_count: blobs.length, blobs: contents }
}

async function fetchBlob(url) {
    const response = await fetch(new URL(`/blob?url=${url}`, window.location.href).href)
    return response.text()
}

function renderScans(repo) {
    let id = repo.tree.id

    return htmx.ajax('POST', '/scans', { target: `#scan${id}`, values: { id: id } });
};

async function putLinks({ name, links }) {
    let formData = new FormData();
    formData.append('action', 3);
    formData.append('links', JSON.stringify(links));
    formData.append('repo_name', name);

    const data = new URLSearchParams(formData)

    var response = await fetch(new URL(`/dashboard`, window.location.href).href, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: data
    })

    return response.json()
};


async function generate({ repo, blobs }) {
    clearCharts()
    setUpWorker(workers)

    let links = await parseBlobs(repo, blobs);
    await putLinks({ name: repo.name, links })

    await renderScans(repo)
    loadLinks(links)
};

async function parseBlobs({ name }, blobs) {
    var currentProgress = name.replace(/\W/g, '');
    $("." + currentProgress).attr("helper-text", "Parsing files");

    var worker = workers.pop()

    try {
        const handlers = await Comlink.wrap(worker).handlers;
        let handler = handlers["multi"]
        let { rawImageData, time } = await handler({
            blobs
        });
        rawImageData = rawImageData.flatMap(el => el)

        $("." + currentProgress).attr("helper-text", "All done");
        $("." + currentProgress).attr("status", "finished");
        $("." + currentProgress).removeClass('currentProgress')

        await putStat({ name: name, task: "parsed (edges)", value: rawImageData.length })
        await putStat({ name: name, task: "speed (ms)", value: (time).toFixed() })

        return rawImageData
    } catch (error) {
        $("." + currentProgress).attr("helper-text", "Parsing error");
        $("." + currentProgress).attr("status", "error");
        $("." + currentProgress).removeClass('currentProgress')
    }



};

function loadLinks(links) {
    // create runtimes 
    const runtimes = [new Runtime(Object.assign(new Library, { links: links })), new Runtime(Object.assign(new Library, { links: links }))]

    // TODO try with structured clones 
    nlink(structuredClone(links))
    tlink(structuredClone(links))

    runtimes[0].module(ndefine, Inspector.into(document.querySelector("#networks")));
    runtimes[1].module(tdefine, Inspector.into(document.querySelector("#treemap-container")));
};

function renderStats(stats) {

    var gha = document.getElementById("stats").clientWidth;
    var runtime = new Runtime(Object.assign(new Library, { stats: stats, width: gha }));
    stats.sort((a, b) => a.task.localeCompare(b.task))
    runtime.module(define, Inspector.into(document.querySelector("#stats")));
}


