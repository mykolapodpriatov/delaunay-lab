# delaunay-lab

Incremental **Delaunay triangulation** of a point cloud, plus the dual **Voronoi** diagram.

This is not `seidel-triangulate`. Seidel cuts a *polygon* (holes allowed). This lab connects *sites*.

- Rust kernel: integer `orient` / `InCircle`, Bowyer–Watson insertion, hull, Voronoi dual
- Browser: click sites, toggle layers, generate uniform / circle / cluster clouds

```bash
cargo test
cd web && npm install && npm run dev
```

Provenance: NTU KhPI computational-geometry notes (incremental Delaunay, Delaunay–Voronoi–hull duality). Rewritten. MIT.
