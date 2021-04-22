
const result = await Deno.emit("src/bundle.ts", {
  bundle: "esm",
  importMapPath: "file:///import-map.json",
  importMap: JSON.parse(Deno.readTextFileSync('./import_map.json')),
  compilerOptions: JSON.parse(Deno.readTextFileSync('./tsconfig.json')).compilerOptions,
});
console.info(result.files["file:///var/www/tauri/tooling/api/src/bundle.ts.js"]);
