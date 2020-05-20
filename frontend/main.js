import init, { run_app } from "./pkg/app.js";
async function main() {
  await init("/pkg/app_bg.wasm");
  run_app();
}
main();

