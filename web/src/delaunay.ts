export type Pt = { x: number; y: number };

export type Mesh = {
  points: Pt[];
  triangles: [number, number, number][];
  hull: Pt[];
  voronoi: [Pt, Pt][];
};

function orient(a: Pt, b: Pt, c: Pt): number {
  return (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
}

function incircle(a: Pt, b: Pt, c: Pt, d: Pt): number {
  const adx = a.x - d.x;
  const ady = a.y - d.y;
  const bdx = b.x - d.x;
  const bdy = b.y - d.y;
  const cdx = c.x - d.x;
  const cdy = c.y - d.y;
  const abdet = adx * bdy - bdx * ady;
  const bcdet = bdx * cdy - cdx * bdy;
  const cadet = cdx * ady - adx * cdy;
  return (
    (adx * adx + ady * ady) * bcdet +
    (bdx * bdx + bdy * bdy) * cadet +
    (cdx * cdx + cdy * cdy) * abdet
  );
}

function ccw(pts: Pt[], a: number, b: number, c: number): [number, number, number] {
  return orient(pts[a], pts[b], pts[c]) < 0 ? [a, c, b] : [a, b, c];
}

export function convexHull(points: Pt[]): Pt[] {
  const pts = [...points].sort((p, q) => p.x - q.x || p.y - q.y);
  if (pts.length <= 2) return pts;
  const lower: Pt[] = [];
  for (const p of pts) {
    while (lower.length >= 2 && orient(lower[lower.length - 2], lower[lower.length - 1], p) <= 0) {
      lower.pop();
    }
    lower.push(p);
  }
  const upper: Pt[] = [];
  for (let i = pts.length - 1; i >= 0; i--) {
    const p = pts[i];
    while (upper.length >= 2 && orient(upper[upper.length - 2], upper[upper.length - 1], p) <= 0) {
      upper.pop();
    }
    upper.push(p);
  }
  lower.pop();
  upper.pop();
  return lower.concat(upper);
}

export function triangulate(input: Pt[]): Mesh {
  const points: Pt[] = [];
  for (const p of input) {
    if (!points.some((q) => q.x === p.x && q.y === p.y)) points.push(p);
  }
  if (points.length < 3) {
    return { points, triangles: [], hull: convexHull(points), voronoi: [] };
  }
  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys);
  const d = Math.max(maxX - minX, maxY - minY, 1);
  const midX = (minX + maxX) / 2;
  const midY = (minY + maxY) / 2;
  const i1 = points.length;
  points.push({ x: midX - 20 * d, y: midY - d });
  points.push({ x: midX + 20 * d, y: midY - d });
  points.push({ x: midX, y: midY + 20 * d });
  let tris: [number, number, number][] = [ccw(points, i1, i1 + 1, i1 + 2)];
  const nReal = i1;
  for (let pi = 0; pi < nReal; pi++) {
    const p = points[pi];
    const bad: number[] = [];
    tris.forEach((t, ti) => {
      if (incircle(points[t[0]], points[t[1]], points[t[2]], p) > 0) bad.push(ti);
    });
    const edges = new Map<string, { a: number; b: number; n: number }>();
    for (const ti of bad) {
      const t = tris[ti];
      for (let k = 0; k < 3; k++) {
        const a = t[k];
        const b = t[(k + 1) % 3];
        const key = a < b ? `${a}-${b}` : `${b}-${a}`;
        const cur = edges.get(key);
        if (cur) cur.n += 1;
        else edges.set(key, { a, b, n: 1 });
      }
    }
    const next = tris.filter((_, i) => !bad.includes(i));
    for (const e of edges.values()) {
      if (e.n !== 1) continue;
      if (orient(points[e.a], points[e.b], p) === 0) continue;
      next.push(ccw(points, e.a, e.b, pi));
    }
    tris = next;
  }
  const triangles = tris.filter((t) => t.every((v) => v < nReal));
  const real = points.slice(0, nReal);
  return {
    points: real,
    triangles,
    hull: convexHull(real),
    voronoi: voronoi(real, triangles),
  };
}

function circumcenter(a: Pt, b: Pt, c: Pt): Pt | null {
  const d = 2 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));
  if (Math.abs(d) < 1e-12) return null;
  const a2 = a.x * a.x + a.y * a.y;
  const b2 = b.x * b.x + b.y * b.y;
  const c2 = c.x * c.x + c.y * c.y;
  return {
    x: (a2 * (b.y - c.y) + b2 * (c.y - a.y) + c2 * (a.y - b.y)) / d,
    y: (a2 * (c.x - b.x) + b2 * (a.x - c.x) + c2 * (b.x - a.x)) / d,
  };
}

function voronoi(points: Pt[], triangles: [number, number, number][]): [Pt, Pt][] {
  const centers = triangles.map((t) => circumcenter(points[t[0]], points[t[1]], points[t[2]]));
  const map = new Map<string, number[]>();
  triangles.forEach((t, ti) => {
    for (let k = 0; k < 3; k++) {
      const a = t[k];
      const b = t[(k + 1) % 3];
      const key = a < b ? `${a}-${b}` : `${b}-${a}`;
      const list = map.get(key) ?? [];
      list.push(ti);
      map.set(key, list);
    }
  });
  const edges: [Pt, Pt][] = [];
  for (const list of map.values()) {
    if (list.length === 2) {
      const c0 = centers[list[0]];
      const c1 = centers[list[1]];
      if (c0 && c1) edges.push([c0, c1]);
    }
  }
  return edges;
}

export function generate(kind: "uniform" | "circle" | "clusters", n: number, w: number, h: number, seed: number): Pt[] {
  let s = seed | 1;
  const unit = () => {
    s = Math.imul(s, 1664525) + 1013904223;
    return ((s >>> 0) % 10000) / 10000;
  };
  if (kind === "circle") {
    return Array.from({ length: n }, (_, i) => {
      const t = (Math.PI * 2 * i) / n;
      return { x: w / 2 + 0.38 * w * Math.cos(t), y: h / 2 + 0.38 * h * Math.sin(t) };
    });
  }
  if (kind === "clusters") {
    const cs = [
      [w * 0.28, h * 0.3],
      [w * 0.72, h * 0.32],
      [w * 0.5, h * 0.72],
    ];
    return Array.from({ length: n }, (_, i) => {
      const [cx, cy] = cs[i % 3];
      return { x: cx + (unit() - 0.5) * w * 0.18, y: cy + (unit() - 0.5) * h * 0.18 };
    });
  }
  return Array.from({ length: n }, () => ({ x: unit() * w, y: unit() * h }));
}
