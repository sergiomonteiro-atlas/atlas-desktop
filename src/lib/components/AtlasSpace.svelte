<script lang="ts">
  import { onMount } from "svelte";
  import * as THREE from "three";
  import { OrbitControls } from "three/addons/controls/OrbitControls.js";
  import { kindColors, type GraphNode, type GraphPayload, type AtlasEvent } from "$lib/atlas/contracts";
  let { graph, mode = "galaxy", states, onselect }: { graph: GraphPayload; mode?: "galaxy" | "command"; states: Record<string, AtlasEvent>; onselect: (node: GraphNode) => void } = $props();
  let host: HTMLDivElement;
  let failed = $state("");
  let draw: (() => void) | undefined;
  $effect(() => { graph; mode; states; draw?.(); });
  onMount(() => {
    let renderer: THREE.WebGLRenderer;
    try { renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true }); } catch (error) { failed = `WebGL indisponível: ${String(error)}`; return; }
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2)); host.appendChild(renderer.domElement);
    const scene = new THREE.Scene(); scene.fog = new THREE.FogExp2(0x080e1a, .012);
    const camera = new THREE.PerspectiveCamera(48, 1, .1, 500); camera.position.set(0, 23, 55);
    const controls = new OrbitControls(camera, renderer.domElement); controls.enableDamping = true; controls.minDistance = 7; controls.maxDistance = 130;
    scene.add(new THREE.AmbientLight(0xabc5ff, 1.4)); const key = new THREE.PointLight(0xffdc99, 100, 130); key.position.set(0, 20, 12); scene.add(key);
    const grid = new THREE.GridHelper(160, 40, 0x223e54, 0x122437); grid.position.y = -8; scene.add(grid);
    let group = new THREE.Group(); scene.add(group); let meshes: THREE.Mesh[] = []; let tracked: { mesh: THREE.Mesh; slug: string }[] = []; let connections: { line: THREE.Line; source: string; target: string }[] = [];
    const release = () => { group.traverse(o => { if (o instanceof THREE.Mesh || o instanceof THREE.Line) { o.geometry.dispose(); const materials = Array.isArray(o.material) ? o.material : [o.material]; materials.forEach(m => m.dispose()); } }); scene.remove(group); group = new THREE.Group(); scene.add(group); meshes = []; tracked = []; connections = []; };
    const ray = new THREE.Raycaster(); const pointer = new THREE.Vector2(); let down = { x: 0, y: 0 };
    const pick = (event: PointerEvent) => { if (Math.hypot(event.clientX - down.x, event.clientY - down.y) > 5) return; const rect = renderer.domElement.getBoundingClientRect(); pointer.set((event.clientX - rect.left) / rect.width * 2 - 1, -(event.clientY - rect.top) / rect.height * 2 + 1); ray.setFromCamera(pointer, camera); const hit = ray.intersectObjects(meshes)[0]; if (hit?.object.userData.node) onselect(hit.object.userData.node as GraphNode); };
    const begin = (event: PointerEvent) => { down = { x: event.clientX, y: event.clientY }; };
    renderer.domElement.addEventListener("pointerdown", begin); renderer.domElement.addEventListener("pointerup", pick);
    draw = () => { release(); const visible = mode === "command" ? graph.nodes.filter(n => n.kind === "agent") : graph.nodes; const limit = visible.slice(0, 1500); const positions = new Map<string, THREE.Vector3>();
      limit.forEach((node, index) => { const angle = index * 2.399963; const radius = 7 + Math.sqrt(index) * 2.05; const pos = mode === "command" ? new THREE.Vector3((index % 7 - 3) * 7, -5, (Math.floor(index / 7) - Math.floor(limit.length / 14)) * 7) : new THREE.Vector3(Math.cos(angle) * radius, Math.sin(index * 1.7) * (5 + Math.sqrt(index)), Math.sin(angle) * radius); positions.set(node.id, pos);
        const color = kindColors[node.kind]; const body = new THREE.Mesh(mode === "command" ? new THREE.CapsuleGeometry(.55, 1.5, 3, 8) : new THREE.SphereGeometry(node.kind === "agent" ? .75 : .45, 12, 8), new THREE.MeshStandardMaterial({ color, emissive: color, emissiveIntensity: .4, roughness: .35, metalness: .5 })); body.position.copy(pos); body.userData.node = node; group.add(body); meshes.push(body); tracked.push({ mesh: body, slug: node.path.split("/").pop()!.replace(/\.md$/, "") });
        if (mode === "command") { const desk = new THREE.Mesh(new THREE.BoxGeometry(4, .3, 2.5), new THREE.MeshStandardMaterial({ color: 0x20374a, metalness: .6 })); desk.position.copy(pos).add(new THREE.Vector3(0, -1, 1.4)); group.add(desk); const screen = new THREE.Mesh(new THREE.BoxGeometry(1.7, 1, .15), new THREE.MeshStandardMaterial({ color: 0x49d6dc, emissive: 0x49d6dc, emissiveIntensity: .6 })); screen.position.copy(pos).add(new THREE.Vector3(0, -.2, 2)); group.add(screen); }
      });
      if (mode === "galaxy") graph.links.forEach(link => { const a = positions.get(link.source), b = positions.get(link.target); if (!a || !b) return; const line = new THREE.Line(new THREE.BufferGeometry().setFromPoints([a, b]), new THREE.LineBasicMaterial({ color: 0x58a4bb, transparent: true, opacity: .25 })); group.add(line); connections.push({line, source:link.source.split("/").pop()!,target:link.target.split("/").pop()!}); });
    }; draw();
    const observer = new ResizeObserver(() => { const w = host.clientWidth, h = host.clientHeight; renderer.setSize(w, h); camera.aspect = w / Math.max(h, 1); camera.updateProjectionMatrix(); }); observer.observe(host);
    let frame = 0; const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const animate = (time: number) => { frame = requestAnimationFrame(animate); controls.update(); tracked.forEach(({ mesh, slug }) => { const state = states[slug]; const recent = state && Date.now() - Date.parse(state.timestamp) < 30000; const active = recent && state.type !== "agent:done" && state.type !== "agent:error"; const material = mesh.material as THREE.MeshStandardMaterial; material.emissiveIntensity = active ? .9 + (reduced ? 0 : Math.sin(time * .006) * .5) : .35; if (recent && state.type === "agent:critic_review") material.emissive.set(0xc39bf5); else if (recent && state.type === "agent:error") material.emissive.set(0xef6571); else material.emissive.set(material.color); mesh.scale.setScalar(active && !reduced ? 1 + Math.sin(time * .004) * .1 : 1); }); connections.forEach(({line,source,target})=> { const active = [states[source],states[target]].some(event => event && Date.now()-Date.parse(event.timestamp)<15000 && !["agent:done","agent:error"].includes(event.type)); const material = line.material as THREE.LineBasicMaterial; material.opacity = active ? .6 + (reduced ? 0 : Math.sin(time*.008)*.25) : .2; material.color.set(active ? 0x8aeaf3 : 0x58a4bb); }); renderer.render(scene, camera); }; frame = requestAnimationFrame(animate);
    return () => { draw = undefined; cancelAnimationFrame(frame); observer.disconnect(); controls.dispose(); renderer.domElement.removeEventListener("pointerdown", begin); renderer.domElement.removeEventListener("pointerup", pick); release(); grid.geometry.dispose(); (grid.material as THREE.Material).dispose(); renderer.dispose(); renderer.domElement.remove(); };
  });
</script>
<div class="space" bind:this={host} role="img" aria-label={mode === "galaxy" ? "Grafo tridimensional do Vault; os mesmos documentos estão disponíveis na lista acessível" : "Estações tridimensionais dos agentes com telemetria de execução"}>
  {#if failed}<p class="failure">{failed}<br />Explore os documentos pela lista lateral.</p>{/if}
</div>
<style>.space{width:100%;height:100%;min-height:360px;position:relative}.space :global(canvas){display:block;width:100%;height:100%;touch-action:none}.failure{padding:30px;color:#f3b4b4}</style>
