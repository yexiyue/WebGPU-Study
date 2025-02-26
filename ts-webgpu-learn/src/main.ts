import "./style.css";
import init, { hello_world } from "rs-wgpu-learn";

init().then(() => {
  document.querySelector<HTMLDivElement>(
    "#app"
  )!.innerHTML = `<p>${hello_world()}</p>`;
});
