const EVALUATE_KEY = 13;
const NAVIGATE_UP_KEY = 38;
const NAVIGATE_DOWN_KEY = 40;

let output = document.getElementById("output");
let inputText = document.getElementById("input-text");
let inputHint = document.getElementById("input-hint");
let input = document.getElementById("input");
let history = [];
let navigation = 0;

const invoke = window.__TAURI__.invoke;
async function evaluateFendWithTimeout(input, timeout) {
    return invoke("fend_prompt", {"value": input, "timeout": timeout});
}

async function evaluateFendPreviewWithTimeout(input, timeout) {
    return invoke("fend_preview_prompt", {"value": input, "timeout": timeout});
}

const setHintInnerText = x => {
    inputHint.innerText = x;
};

async function commands(event) {
    if (!event.ctrlKey) {
        return;
    }

    if (event.key === "w") {
        invoke("quit");
    } else if (event.key === "c" && document.getSelection().isCollapsed) {
        invoke("copy_to_clipboard", {"value": inputText.value})
        // TODO; optionally make this copy other values, such as the hint or the previous output
        // Also add a toast (https://www.w3schools.com/howto/howto_js_snackbar.asp)
    } else if (event.key === "s") {
        invoke("save_to_file")
    } else if (event.key === "o") {
        // TODO
    }
}

async function evaluate(event) {
    // allow multiple lines to be entered if shift, ctrl
    // or meta is held, otherwise evaluate the expression
    if (!(event.keyCode === EVALUATE_KEY && !event.shiftKey && !event.ctrlKey && !event.metaKey)) {
        return;
    }

    event.preventDefault();

    if (inputText.value === "clear") {
        output.innerHTML = "";
        setInputText("");
        inputHint.innerText = "";
        return;
    }

    let request = document.createElement("p");
    let result = document.createElement("p");

    request.innerText = "> " + inputText.value;

    if (isInputFilled()) {
        history.push(inputText.value);
    }

    navigateEnd();

    const setResultInnerText = x => {
        result.innerText = x;
    };

    evaluateFendWithTimeout(inputText.value, 500).then(setResultInnerText, setResultInnerText).finally(() => {
        setInputText("");
        setHintInnerText("");
        output.appendChild(request);
        output.appendChild(result);

        result.scrollIntoView();
    });
}

function navigate(event) {
    if (![NAVIGATE_UP_KEY, NAVIGATE_DOWN_KEY].includes(event.keyCode)) {
        return;
    }
    if (navigation > 0) {
        if (NAVIGATE_UP_KEY === event.keyCode) {
            event.preventDefault();

            navigateBackwards();
        }

        else if (NAVIGATE_DOWN_KEY === event.keyCode) {
            event.preventDefault();

            navigateForwards();
        }

    } else if (!isInputFilled() && history.length > 0 && NAVIGATE_UP_KEY === event.keyCode) {
        event.preventDefault();

        navigateBegin();
    }

    if (navigation > 0) {
        navigateSet();
    }

    updateReplicatedText();
    updateHint();
}

function navigateBackwards() {
    navigation += 1;

    if (navigation > history.length) {
        navigation = history.length;
    }
}

function navigateForwards() {
    navigation -= 1;

    if (navigation < 1) {
        navigateEnd();
        navigateClear();
    }
}

function navigateBegin() {
    navigation = 1;
}

function navigateEnd() {
    navigation = 0;
}

function navigateSet() {
    setInputText(history[history.length - navigation]);
}

function navigateClear() {
    setInputText("");
}

function focus() {
    // allow the user to select text for copying and
    // pasting, but if text is deselected (collapsed)
    // refocus the input field
    if (document.activeElement !== inputText && document.getSelection().isCollapsed) {
        inputText.focus();
    }
}

async function update() {
    updateReplicatedText();
    navigateEnd();
    updateHint();
}

function updateReplicatedText() {
    inputText.parentNode.dataset.replicatedValue = inputText.value;
}

function updateHint() {
    evaluateFendPreviewWithTimeout(inputText.value, 100).then(x => {
        inputHint.className = "valid-hint";
        setHintInnerText(x);
    }, x => {
        inputHint.className = "error-hint";
        setHintInnerText(x);
    })
}

function isInputFilled() {
    return inputText.value.length > 0;
}

function setInputText(x) {
    inputText.value = x;
}

async function load() {
    const keydown = x => {
        navigate(x);
        evaluate(x);
        commands(x);
    }
    inputText.addEventListener('input', update);
    inputText.addEventListener('keydown', keydown);
    document.addEventListener('click', focus)
}

window.onload = load;
