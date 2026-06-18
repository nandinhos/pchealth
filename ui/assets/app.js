// PBHealth — lógica do frontend
//
// Usa o objeto `window.__TAURI__` para invocar comandos Rust.
// Sem framework JS (sem React/Vue) — vanilla JS puro.

(function () {
    "use strict";

    // ── Tauri bridge ──────────────────────────────────────────
    // Tauri 2 expõe invoke() via window.__TAURI__.core.invoke()
    const invoke = window.__TAURI__?.core?.invoke
        || window.__TAURI__?.invoke
        || (async () => { throw new Error("Tauri não disponível — rodando no browser?"); });

    // ── Elementos ─────────────────────────────────────────────
    const btnRefresh = document.getElementById("btn-refresh");
    const cardOverall = document.getElementById("card-overall");
    const cardLoading = document.getElementById("card-loading");
    const cardError = document.getElementById("card-error");
    const errorMessage = document.getElementById("error-message");
    const cardsGrid = document.getElementById("cards-grid");
    const overallScore = document.getElementById("overall-score");
    const overallStatus = document.getElementById("overall-status");
    const overallSummary = document.getElementById("overall-summary");
    const overallMeta = document.getElementById("overall-meta");

    // ── Helpers ────────────────────────────────────────────────
    const STATUS_LABELS_PT = {
        healthy: "Saudável",
        attention: "Atenção",
        critical: "Crítico",
        unavailable: "Indisponível",
    };

    function show(el) { el.classList.remove("hidden"); }
    function hide(el) { el.classList.add("hidden"); }

    function setStatusClass(el, status) {
        el.classList.remove("healthy", "attention", "critical", "unavailable");
        if (status) el.classList.add(status);
    }

    function formatTime(iso) {
        try {
            const d = new Date(iso);
            return d.toLocaleString("pt-BR");
        } catch {
            return iso;
        }
    }

    // ── Render do score geral ──────────────────────────────────
    function renderOverall(report) {
        const { score, status, counts } = report.score;

        // Score circle
        const circle = overallScore.querySelector(".score-circle");
        const valueEl = overallScore.querySelector(".score-value");
        valueEl.textContent = String(score);
        setStatusClass(circle, status);

        // Status text
        overallStatus.textContent = STATUS_LABELS_PT[status] || status;
        setStatusClass(overallStatus, status);

        // Summary
        const parts = [];
        if (counts.critical > 0) parts.push(`${counts.critical} crítica(s)`);
        if (counts.attention > 0) parts.push(`${counts.attention} em atenção`);
        if (counts.healthy > 0) parts.push(`${counts.healthy} normal`);
        if (counts.unavailable > 0) parts.push(`${counts.unavailable} indisponível`);
        overallSummary.textContent = parts.length
            ? parts.join(" · ")
            : "Nenhuma métrica coletada.";

        // Meta
        overallMeta.textContent = `${report.os} · ${counts.total} métricas · ${formatTime(report.timestamp)}`;

        show(cardOverall);
    }

    // ── Render dos cards por categoria ─────────────────────────
    const CATEGORY_LABELS = {
        machine: "Máquina",
        bios: "BIOS / Placa-mãe",
        cpu: "Processador",
        memory: "Memória RAM",
        gpu: "Placa de vídeo",
        storage: "Armazenamento",
        sensors: "Sensores",
        power: "Energia / Bateria",
        network: "Rede",
        os: "Sistema operacional",
    };

    function renderCategoryCard(category, metrics) {
        const card = document.querySelector(`[data-category="${category}"]`);
        if (!card) return;

        const body = card.querySelector(".card-body");
        body.innerHTML = "";

        if (metrics.length === 0) {
            body.innerHTML = '<p class="muted small">Nenhuma métrica nesta categoria.</p>';
            return;
        }

        for (const m of metrics) {
            const row = document.createElement("div");
            row.className = "metric-row";

            const keyEl = document.createElement("span");
            keyEl.className = "metric-key";
            keyEl.textContent = m.key;

            const valueEl = document.createElement("span");
            valueEl.className = `metric-value status-${m.status}`;
            valueEl.textContent = m.value ?? "—";

            const badge = document.createElement("span");
            badge.className = `badge ${m.status}`;
            badge.textContent = STATUS_LABELS_PT[m.status] || m.status;

            valueEl.appendChild(badge);

            row.appendChild(keyEl);
            row.appendChild(valueEl);
            body.appendChild(row);
        }
    }

    function renderAllCategories(metrics) {
        const grouped = {};

        for (const m of metrics) {
            const cat = m.category;
            if (!grouped[cat]) grouped[cat] = [];
            grouped[cat].push(m);
        }

        // Limpa cards antes de popular
        document.querySelectorAll(".cards-grid .card").forEach((card) => {
            card.classList.add("hidden");
        });

        for (const [category, items] of Object.entries(grouped)) {
            const card = document.querySelector(`[data-category="${category}"]`);
            if (card) {
                card.classList.remove("hidden");
                renderCategoryCard(category, items);
            }
        }

        show(cardsGrid);
    }

    // ── Pipeline principal ────────────────────────────────────
    async function runDiagnostic() {
        hide(cardOverall);
        hide(cardsGrid);
        hide(cardError);
        show(cardLoading);
        btnRefresh.disabled = true;
        btnRefresh.textContent = "⏳ Coletando...";

        try {
            const report = await invoke("run_diagnostic");

            hide(cardLoading);
            renderOverall(report);
            renderAllCategories(report.metrics);
        } catch (err) {
            hide(cardLoading);
            errorMessage.textContent = String(err);
            show(cardError);
            console.error("run_diagnostic falhou:", err);
        } finally {
            btnRefresh.disabled = false;
            btnRefresh.textContent = "🔄 Diagnosticar agora";
        }
    }

    // ── Setup inicial ─────────────────────────────────────────
    btnRefresh.addEventListener("click", runDiagnostic);

    document.getElementById("year").textContent = new Date().getFullYear();

    // Auto-refresh opcional virá na Fase 7 — por enquanto só manual.
    console.log("PBHealth UI pronta. Tauri disponível:", !!window.__TAURI__);
})();