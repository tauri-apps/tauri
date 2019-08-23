<template>
  <div>
    <section class="page-header fixed-top shadow-8 scroll-determined" style="" v-scroll="scrolled">
      <div class="bg-container scroll-determined q-pa-md"></div>
      <div>
        <div>
          <img src="statics/tauri-text.png" class="tauri-name scroll-determined">
        </div>
        <div v-if="buttons" style="margin-top:100px">
          <q-btn type="a" href="https://github.com/quasarframework/tauri" target="_blank" class="btn" label="GitHub" no-caps flat/>
          <q-btn to="/examples" class="btn" label="Examples" no-caps flat color="warning" text-color="black"/>
          <q-btn type="a" href="https://donate.quasar.dev" target="_blank" class="btn" label="Donate" no-caps flat/>
        </div>
      </div>
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
  name: 'Hero',
  data () {
    return {
      buttons: true,
      height: 270,
      heightName: 140,
      heightPic: 250
    }
  },
  methods: {
    scrolled (position) {
      this.height = 270 - (position / 4)
      this.heightName = 140 - (position / 4)
      this.heightPic = 250 - (position / 4)
      if (this.height <= 90) {
        this.height = 90
        this.heightPic = 90
        this.buttons = false
      } else {
        this.buttons = true
      }
      if (this.heightName <= 50) {
        this.heightName = 50
      }
      console.log(this.heightName)
      document.getElementsByClassName('scroll-determined')[0].setAttribute('style', `height: ${this.height}px`)
      document.getElementsByClassName('scroll-determined')[1].setAttribute('style', `height: ${this.heightPic}px;width: ${this.heightPic}px;transform: rotate(${position}deg)`)
      document.getElementsByClassName('tauri-name')[0].setAttribute('style', `
      height: ${this.heightName}px;
      `)
    }
  }
}
</script>

<style lang="stylus">
.tauri-name
  max-height 100px
  min-height 20px
  position fixed
  margin auto
  top 25px
  left -10px
  right 0
.page-header
  height 270px
  z-index 1000000
  border-bottom 2px solid #212111
.bg-container
  background-image url(https://cdn.quasar.dev/logo/tauri/tauri-logo-240x240.png)
  background-repeat no-repeat
  background-size contain
  opacity 0.2
  position absolute
  left 0
  top 5px
  height 250px
  width 250px
  max-height 250px
  max-width 250px
</style>
