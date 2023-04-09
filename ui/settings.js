const invoke = window.__TAURI__.invoke;

async function apply_settings() {
    invoke("get_settings").then(x => {
        for (let key in x) {
            let input = document.getElementById(key);

            if (input === null) {
                console.error("Unknown settings key: " + key);
                continue;
            }

            if (input.getAttribute("type") === "checkbox") {
                input.checked = x[key];
            } else {
                input.value = x[key];
            }
        }
    })
}

(function() {
    let ranges = document.querySelectorAll("input, select");
    for (let i = 0; i < ranges.length; i++) {
        if (ranges[i].getAttribute("type") === "checkbox") {
            ranges[i].addEventListener("input", function (e) {
                invoke("set_setting", {"id": ranges[i].id, "value": e.target.checked})
            });
        } else {
            ranges[i].addEventListener("input", function (e) {
                invoke("set_setting", {"id": ranges[i].id, "value": e.target.value})
            });
        }
    }

    apply_settings();
})();