const telegram = window.Telegram?.WebApp;
if (telegram) {
  telegram.ready();
  telegram.expand();
}

const state = {
  chatId: null,
  initData: telegram?.initData || "",
  snapshot: null,
};

const errorBox = document.getElementById("errorBox");
const chatTitle = document.getElementById("chatTitle");
const participantsCount = document.getElementById("participantsCount");
const spendsCount = document.getElementById("spendsCount");
const sessionId = document.getElementById("sessionId");
const currentUser = document.getElementById("currentUser");
const participantsList = document.getElementById("participantsList");
const balancesList = document.getElementById("balancesList");
const transfersList = document.getElementById("transfersList");

const spendDialog = document.getElementById("spendDialog");
const totalInput = document.getElementById("totalInput");
const modeSelect = document.getElementById("modeSelect");
const payerSelect = document.getElementById("payerSelect");
const splitRows = document.getElementById("splitRows");
const splitHint = document.getElementById("splitHint");

function showError(message) {
  errorBox.textContent = message;
  errorBox.classList.remove("hidden");
}

function clearError() {
  errorBox.textContent = "";
  errorBox.classList.add("hidden");
}

function formatCents(cents) {
  const sign = cents < 0 ? "-" : "";
  const value = Math.abs(cents);
  const whole = Math.floor(value / 100);
  const fractional = String(value % 100).padStart(2, "0");
  return `${sign}${whole}.${fractional}`;
}

function row(label, value) {
  return `<div class="list-row"><span>${label}</span><strong>${value}</strong></div>`;
}

function renderSnapshot(snapshot) {
  state.snapshot = snapshot;
  chatTitle.textContent = snapshot.chat.title || `Chat ${snapshot.chat.chat_id}`;
  participantsCount.textContent = String(snapshot.participants.length);
  spendsCount.textContent = String(snapshot.spends_count);
  sessionId.textContent = String(snapshot.session.id);
  currentUser.textContent = `You: ${snapshot.participant.display_name}`;

  participantsList.innerHTML = snapshot.participants
    .map((participant) => row(participant.display_name, `#${participant.user_id}`))
    .join("");

  balancesList.innerHTML = snapshot.balances
    .map((balance) => {
      const prefix = balance.net_cents > 0 ? "+" : "";
      return row(balance.display_name, `${prefix}${formatCents(balance.net_cents)}`);
    })
    .join("");

  if (snapshot.transfers.length === 0) {
    transfersList.innerHTML = `<div class="list-row"><span>No transfers needed</span><strong>Settled</strong></div>`;
  } else {
    transfersList.innerHTML = snapshot.transfers
      .map((transfer) =>
        row(`${transfer.from_name} -> ${transfer.to_name}`, formatCents(transfer.amount_cents))
      )
      .join("");
  }
}

async function postJson(url, body) {
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(payload.error || "Request failed");
  }
  return payload;
}

function populateSpendForm() {
  if (!state.snapshot) {
    return;
  }

  payerSelect.innerHTML = "";
  splitRows.innerHTML = "";
  splitHint.textContent =
    modeSelect.value === "ABS"
      ? "Split values must sum exactly to the total."
      : "Percent values must sum exactly to 100.";

  state.snapshot.participants.forEach((participant) => {
    const option = document.createElement("option");
    option.value = String(participant.user_id);
    option.textContent = participant.display_name;
    if (participant.user_id === state.snapshot.participant.user_id) {
      option.selected = true;
    }
    payerSelect.appendChild(option);

    const wrapper = document.createElement("div");
    wrapper.className = "split-row";

    const label = document.createElement("label");
    label.textContent = participant.display_name;

    const input = document.createElement("input");
    input.type = "text";
    input.placeholder = modeSelect.value === "ABS" ? "0.00" : "0";
    input.dataset.userId = String(participant.user_id);

    wrapper.append(label, input);
    splitRows.appendChild(wrapper);
  });
}

function equalSplit() {
  const inputs = Array.from(splitRows.querySelectorAll("input"));
  if (inputs.length === 0) {
    return;
  }

  if (modeSelect.value === "ABS") {
    const total = totalInput.value.trim();
    if (!/^\d+(\.\d{1,2})?$/.test(total)) {
      showError("Enter a valid total before using equal split.");
      return;
    }

    const parts = total.split(".");
    const cents = (parseInt(parts[0], 10) * 100) + parseInt((parts[1] || "0").padEnd(2, "0"), 10);
    const base = Math.floor(cents / inputs.length);
    let remainder = cents - (base * inputs.length);

    inputs.forEach((input) => {
      let value = base;
      if (remainder > 0) {
        value += 1;
        remainder -= 1;
      }
      input.value = formatCents(value);
    });
  } else {
    const basisPoints = 10_000;
    const base = Math.floor(basisPoints / inputs.length);
    let remainder = basisPoints - (base * inputs.length);

    inputs.forEach((input) => {
      let value = base;
      if (remainder > 0) {
        value += 1;
        remainder -= 1;
      }
      input.value = (value / 100).toFixed(2);
    });
  }

  clearError();
}

async function bootstrap() {
  if (!state.initData) {
    showError("This Mini App must be opened from Telegram. Browser fallback can load the page, but backend actions require signed Telegram init data.");
    return;
  }

  const snapshot = await postJson("/api/bootstrap", {
    chat_id: state.chatId,
    init_data: state.initData,
  });
  renderSnapshot(snapshot);
}

async function refreshReport() {
  const snapshot = await postJson("/api/report", {
    chat_id: state.chatId,
    init_data: state.initData,
  });
  renderSnapshot(snapshot);
}

async function submitSpend() {
  const splits = Array.from(splitRows.querySelectorAll("input")).map((input) => ({
    user_id: Number(input.dataset.userId),
    value: input.value.trim(),
  }));

  const snapshot = await postJson("/api/spends", {
    chat_id: state.chatId,
    init_data: state.initData,
    total: totalInput.value.trim(),
    mode: modeSelect.value,
    payer_user_id: Number(payerSelect.value),
    splits,
  });

  spendDialog.close();
  renderSnapshot(snapshot);
}

async function resetSession() {
  if (!window.confirm("Reset the current trip session? The bot will post the current report first.")) {
    return;
  }

  const snapshot = await postJson("/api/reset", {
    chat_id: state.chatId,
    init_data: state.initData,
  });
  renderSnapshot(snapshot);
}

function init() {
  const params = new URLSearchParams(window.location.search);
  state.chatId = Number(params.get("chat_id"));
  if (!state.chatId) {
    showError("Missing chat_id in URL.");
    return;
  }

  document.getElementById("refreshButton").addEventListener("click", async () => {
    try {
      clearError();
      await refreshReport();
    } catch (error) {
      showError(error.message);
    }
  });

  document.getElementById("addSpendButton").addEventListener("click", () => {
    if (!state.snapshot) {
      return;
    }
    totalInput.value = "";
    modeSelect.value = "ABS";
    populateSpendForm();
    spendDialog.showModal();
  });

  document.getElementById("closeDialogButton").addEventListener("click", () => {
    spendDialog.close();
  });
  document.getElementById("equalSplitButton").addEventListener("click", equalSplit);
  document.getElementById("modeSelect").addEventListener("change", populateSpendForm);
  document.getElementById("submitSpendButton").addEventListener("click", async () => {
    try {
      clearError();
      await submitSpend();
    } catch (error) {
      showError(error.message);
    }
  });
  document.getElementById("resetButton").addEventListener("click", async () => {
    try {
      clearError();
      await resetSession();
    } catch (error) {
      showError(error.message);
    }
  });

  bootstrap().catch((error) => showError(error.message));
}

init();
