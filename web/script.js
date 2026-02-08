const API = window.location.origin;

// Tabs
document.querySelectorAll(".tab").forEach((tab) => {
	tab.addEventListener("click", () => {
		document.querySelectorAll(".tab").map((t) => t.classList.remove("active"));
		document
			.querySelectorAll(".panel")
			.map((p) => p.classList.remove("active"));
		tab.classList.add("active");
		document.getElementById(`panel-${tab.dataset.tab}`).classList.add("active");
	});
});

async function api(body) {
	const res = await fetch(API, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(body),
	});
	return res.json();
}

async function doTo() {
	const path = document.getElementById("to-path").value;
	const format = document.getElementById("to-format").value;
	const result = document.getElementById("to-result");
	const value = document.getElementById("to-value");

	if (!path) return;

	const data = await api({ action: "to", path, format });
	result.classList.add("show");
	result.classList.toggle("success", data.success);
	result.classList.toggle("error", !data.success);
	value.textContent = data.success ? data.fsid : data.error;
}

async function doFrom() {
	const fsid = document.getElementById("from-fsid").value;
	const result = document.getElementById("from-result");
	const value = document.getElementById("from-value");

	if (!fsid) return;

	const data = await api({ action: "from", fsid });
	result.classList.add("show");
	result.classList.toggle("success", data.success);
	result.classList.toggle("error", !data.success);
	value.textContent = data.success ? data.path : data.error;
}

async function doInfo() {
	const fsid = document.getElementById("info-fsid").value;
	const result = document.getElementById("info-result");
	const value = document.getElementById("info-value");

	if (!fsid) return;

	const data = await api({ action: "info", fsid });
	result.classList.add("show");
	result.classList.toggle("success", data.success);
	result.classList.toggle("error", !data.success);

	if (data.success && data.info) {
		const i = data.info;
		value.innerHTML = `
          <span class="info-label">FSID</span><span class="info-value">${i.fsid}</span>
          <span class="info-label">Format</span><span class="info-value">${i.format}</span>
          <span class="info-label">Prefix</span><span class="info-value">${i.prefix} (${i.prefix_code})</span>
          <span class="info-label">Type</span><span class="info-value">${i.file_type}</span>
          <span class="info-label">Mode</span><span class="info-value">${i.mode}</span>
          <span class="info-label">Valid</span><span class="info-value">${i.valid ? "✓" : "✗"}</span>
          <span class="info-label">Path</span><span class="info-value">${i.path || "—"}</span>
        `;
	} else {
		value.innerHTML = `<span class="info-value" style="color:var(--error)">${data.error}</span>`;
	}
}

async function doList() {
	const value = document.getElementById("list-value");
	const data = await api({ action: "list" });

	if (data.success && data.list && data.list.length > 0) {
		value.innerHTML = data.list
			.map(
				(item) => `
          <div class="list-row">
            <span class="list-fsid">${item.fsid}</span>
            <span class="list-path">${item.path}</span>
          </div>
        `,
			)
			.join("");
	} else {
		value.innerHTML = '<div class="empty">No FSIDs registered</div>';
	}
}

// Load list on start
doList();
