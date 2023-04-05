const invoke = window.__TAURI__.invoke;

(function() {
    let ranges = document.querySelectorAll("input");
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
})();