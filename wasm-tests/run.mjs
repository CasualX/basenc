import { readFileSync } from "node:fs";
import { performance } from "node:perf_hooks";

async function load(name) {
	const path = new URL(`target/${name}/wasm32-unknown-unknown/release/basenc_wasm_tests.wasm`, import.meta.url);
	const bytes = readFileSync(path);
	const { instance } = await WebAssembly.instantiate(bytes, {});
	return instance.exports;
}

const modules = new Map([
	["scalar", await load("scalar")],
	["simd", await load("simd")],
]);

let fingerprint;
for (const [name, wasm] of modules) {
	const checks = wasm.verify();
	const expected = wasm.expected_checks() >>> 0;
	if (checks !== expected) throw new Error(`${name}: checks=${checks}, expected=${expected}`);
	const current = wasm.fingerprint() >>> 0;
	if (fingerprint !== undefined && current !== fingerprint) {
		throw new Error(`${name}: fingerprint=${current}, scalar=${fingerprint}`);
	}
	fingerprint = current;
	console.log(`${name}: all checks passed; fingerprint=${current}`);
}

if (process.argv.includes("--bench")) {
	for (const [name, wasm] of modules) {
		const results = [];
		for (const fn of [
			"bench_base64_encode",
			"bench_base64_decode",
			"bench_generic_base64_encode",
			"bench_generic_base64_decode",
			"bench_hex_encode",
		]) {
			for (let warmup = 0; warmup < 5; warmup++) wasm[fn](100);
			const iterations = 10000;
			const start = performance.now();
			const checksum = wasm[fn](iterations);
			const elapsed = performance.now() - start;
			const throughput = 4096 * iterations / elapsed / 1e3;
			results.push(`${fn}=${throughput.toFixed(1)} MB/s (${checksum})`);
		}
		console.log(`${name}: ${results.join("; ")}`);
	}
}
