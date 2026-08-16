import { generate, triangulate, type Pt } from "./delaunay";
import { canvasPoint, draw } from "./draw";

const cv = document.querySelector<HTMLCanvasElement>("#cv")!;
const stats = document.querySelector<HTMLParagraphElement>("#stats")!;
const showD = document.querySelector<HTMLInputElement>("#show-delaunay")!;
const showV = document.querySelector<HTMLInputElement>("#show-voronoi")!;
const showH = document.querySelector<HTMLInputElement>("#show-hull")!;

let points: Pt[] = [];

function layers() {
  return { delaunay: showD.checked, voronoi: showV.checked, hull: showH.checked };
}

function render(): void {
  const mesh = triangulate(points);
  draw(cv, mesh, layers());
  stats.textContent = `${mesh.points.length} sites · ${mesh.triangles.length} triangles · hull ${mesh.hull.length}`;
}

cv.addEventListener("click", (ev) => {
  points.push(canvasPoint(cv, ev));
  render();
});

document.querySelectorAll<HTMLButtonElement>("[data-gen]").forEach((btn) => {
  btn.addEventListener("click", () => {
    const kind = btn.dataset.gen as "uniform" | "circle" | "clusters";
    points = generate(kind, 28, cv.width, cv.height, Date.now() % 100000);
    render();
  });
});

document.querySelector("#clear")!.addEventListener("click", () => {
  points = [];
  render();
});

[showD, showV, showH].forEach((el) => el.addEventListener("change", render));
render();
