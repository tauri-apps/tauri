<template>
  <div>
    <img alt="Vue logo" src="./assets/logo.png" />
    <img alt="Tauri logo" height="200" src="./assets/tauri.svg" />
  </div>
  <HelloWorld msg="Hello Vue 3 + TypeScript + Vite + Tauri" />
  <a
    style="color: #42b983;"
    href="https://tauri.studio"
    target="_blank"
    rel="noopener noreferrer"
  >Learn Tauri</a>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import HelloWorld from './components/HelloWorld.vue'

export default defineComponent({
  name: 'App',
  components: {
    HelloWorld
  }
})
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
