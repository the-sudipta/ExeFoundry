const platform = document.querySelector("#platform");
const inputPath = document.querySelector("#inputPath");
const outputPath = document.querySelector("#outputPath");
const iconPath = document.querySelector("#iconPath");
const guiMode = document.querySelector("#guiMode");
const quotePaths = document.querySelector("#quotePaths");
const commandOutput = document.querySelector("#commandOutput");
const iconStatus = document.querySelector("#iconStatus");
const modeStatus = document.querySelector("#modeStatus");
const copyCommand = document.querySelector("#copyCommand");
const batSample = document.querySelector("#batSample");
const downloadSample = document.querySelector("#downloadSample");

const binaries = {
  windows: ".\\exefoundry-windows-x64.exe",
  linux: "./exefoundry-linux-x64",
  macos: "./exefoundry-macos",
};

function quote(value) {
  if (!quotePaths.checked) {
    return value;
  }
  return `"${value.replaceAll('"', '\\"')}"`;
}

function buildCommand() {
  const parts = [
    binaries[platform.value],
    "--input",
    quote(inputPath.value || "tool.bat"),
    "--output",
    quote(outputPath.value || "Tool.exe"),
  ];

  if (iconPath.value.trim()) {
    parts.push("--icon", quote(iconPath.value.trim()));
  }

  if (guiMode.checked) {
    parts.push("--gui");
  }

  commandOutput.textContent = parts.join(" ");
  iconStatus.textContent = iconPath.value.trim()
    ? "Your custom icon will be embedded into the EXE."
    : "Bundled ExeFoundry icon will be embedded.";
  modeStatus.textContent = guiMode.checked ? "GUI app output." : "Console app output.";
}

function downloadTextFile(filename, text) {
  const blob = new Blob([text], { type: "application/x-bat" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

document.querySelectorAll("input, select").forEach((element) => {
  element.addEventListener("input", buildCommand);
  element.addEventListener("change", buildCommand);
});

copyCommand.addEventListener("click", async () => {
  await navigator.clipboard.writeText(commandOutput.textContent);
  copyCommand.textContent = "Copied";
  setTimeout(() => {
    copyCommand.textContent = "Copy";
  }, 1200);
});

downloadSample.addEventListener("click", () => {
  downloadTextFile("hello-exefoundry.bat", batSample.value);
});

buildCommand();
