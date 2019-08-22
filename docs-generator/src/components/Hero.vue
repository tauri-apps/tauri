<template>
  <div>
    <section class="page-header">
      <h1 class="project-name">Tauri</h1>
      <h2 class="project-tagline"></h2>
      <q-btn type="a" href="https://github.com/quasarframework/tauri" target="_blank" class="btn" label="View on GitHub" no-caps flat/>
      <q-btn to="/docs" class="btn" label="Docs" no-caps flat/>
      <q-btn to="/examples" class="btn" label="Examples" no-caps flat/>
      <q-btn type="a" href="https://donate.quasar.dev" target="_blank" class="btn" label="Donate" no-caps flat/>
    </section>
    <main class="flex flex-start justify-center inset-shadow">
      <div class="q-pa-md col-12-sm col-8-md col-6-lg inset-shadow" style="width: 100%; height: 3px;" />
      <div class="q-pa-md col-12-sm col-8-md col-6-lg bg-white shadow-1" style="max-width: 800px; width: 100%;">
        <slot></slot>
      </div>
    </main>
  </div>
</template>

<script>
export default {
  name: 'Hero'
}
</script>
