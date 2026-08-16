import type { Mesh, Pt } from "./delaunay";

export function draw(
  cv: HTMLCanvasElement,
  mesh: Mesh,
  layers: { delaunay: boolean; voronoi: boolean; hull: boolean },
): void {
  const ctx = cv.getContext("2d");
  if (!ctx) return;
  const w = cv.width;
  const h = cv.height;
  ctx.fillStyle = "#0b1020";
  ctx.fillRect(0, 0, w, h);

  if (layers.voronoi) {
    ctx.strokeStyle = "rgba(94, 234, 212, 0.55)";
    ctx.lineWidth = 1.2;
    for (const [a, b] of mesh.voronoi) {
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.stroke();
    }
  }

  if (layers.delaunay) {
    ctx.strokeStyle = "rgba(251, 191, 36, 0.7)";
    ctx.lineWidth = 1.1;
    for (const t of mesh.triangles) {
      const a = mesh.points[t[0]];
      const b = mesh.points[t[1]];
      const c = mesh.points[t[2]];
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.lineTo(c.x, c.y);
      ctx.closePath();
      ctx.stroke();
    }
  }

  if (layers.hull && mesh.hull.length > 1) {
    ctx.strokeStyle = "#f472b6";
    ctx.lineWidth = 2.4;
    ctx.beginPath();
    mesh.hull.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
    ctx.closePath();
    ctx.stroke();
  }

  for (const p of mesh.points) {
    ctx.fillStyle = "#e2e8f0";
    ctx.beginPath();
    ctx.arc(p.x, p.y, 3.2, 0, Math.PI * 2);
    ctx.fill();
  }
}

export function canvasPoint(cv: HTMLCanvasElement, ev: MouseEvent): Pt {
  const r = cv.getBoundingClientRect();
  return {
    x: ((ev.clientX - r.left) * cv.width) / r.width,
    y: ((ev.clientY - r.top) * cv.height) / r.height,
  };
}
