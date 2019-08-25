<template>
  <div class="full-width">
    <q-page-sticky expand class="page-header fixed-top shadow-8 scroll-determined" v-scroll="scrolled">
      <q-chip id="claim" outline dense square icon="star" icon-right="star" class="claim text-weight-light bg-amber-3" style="top: 220px">Build more secure native apps with fast,  tiny binaries.</q-chip>
      <div class="bg-container scroll-determined q-pa-md q-ml-lg"></div>
      <div>
        <div>
          <img src="statics/tauri-text.png" class="tauri-name scroll-determined">
        </div>
        <div v-if="buttons" class="row" style="margin-top:90px">
          <q-btn dense size="small" type="a" href="https://github.com/quasarframework/tauri" target="_blank" class="btn " label="Quick Start" no-caps color="warning" text-color="black"/>
          <q-btn dense size="small" to="/docs/patterns" class="btn" label="Patterns" no-caps  color="warning" text-color="black"/>
        </div>
      <div v-else class="absolute-right" style="margin:15px 30px 0 0;z-index:10000000">
        <q-btn-dropdown dense color="warning" label="Docs" no-caps text-color="black" class="q-mr-lg">
          <q-list color="warning">
            <q-item clickable v-close-popup to="/docs/quick-start">
              <q-item-section>
                <q-item-label>Quick Start</q-item-label>
              </q-item-section>
            </q-item>
            <q-item clickable v-close-popup to="/docs/patterns">
              <q-item-section>
                <q-item-label>Patterns</q-item-label>
              </q-item-section>
            </q-item>
            <q-item clickable v-close-popup to="/docs/environment">
              <q-item-section>
                <q-item-label>Environment</q-item-label>
              </q-item-section>
            </q-item>
            <q-item clickable v-close-popup to="/docs/api">
              <q-item-section>
                <q-item-label>API</q-item-label>
              </q-item-section>
            </q-item>
          </q-list>
        </q-btn-dropdown>
      </div>
      </div>
    </q-page-sticky>
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
      heightPic: 250,
      heightClaim: 100,
      leftDrawerOpen: this.$q.platform.is.desktop,
      rightDrawerOpen: this.$q.platform.is.desktop
    }
  },
  nounted () {
    this.scrolled(document.offset().top)
  },
  methods: {
    scrolled (position) {
      const pos = position / 4
      this.height = 270 - (pos)
      this.heightName = 140 - (pos)
      this.heightPic = 250 - (pos)
      this.heightClaim = 220 - (pos)
      console.log(this.heightPic)
      if (this.height <= 70) {
        this.height = 70
        this.buttons = false
      }
      if (this.heightPic <= 60) {
        this.heightPic = 60
      }
      if (this.height <= 220) {
        this.buttons = false
      } else {
        this.buttons = true
      }
      if (this.heightName <= 35) {
        this.heightName = 35
      }
      if (this.heightClaim <= 58) {
        this.heightClaim = 58
      }
      // todo: cleanup, use vuex, be faster!
      // the buttons
      document.getElementsByClassName('scroll-determined')[0].setAttribute('style', `height: ${this.height}px`)
      // the icon
      document.getElementsByClassName('scroll-determined')[1].setAttribute('style', `height: ${this.heightPic - 5}px;width: ${this.heightPic}px;transform: rotate(${position}deg)`)
      // the name
      document.getElementsByClassName('tauri-name')[0].setAttribute('style', `
      height: ${this.heightName}px;
      `)
      // claim
      document.getElementById('claim').setAttribute('style', `top: ${this.heightClaim}px`)
      // the sidebar
      document.getElementsByClassName('q-drawer__content')[0].setAttribute('style', `margin-top: ${this.height + 10}px`)
    }
  }
}
</script>

<style lang="stylus">
.q-menu
  z-index 1000000
.tauri-name
  max-height 100px
  min-height 20px
  max-width 50%
  height inherit
  position fixed
  margin auto
  top 15px
  left 0
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
  min-height 60px
  min-width 60px
.claim
  position absolute
  margin 0 auto
  left 20px
  right 20px
  width 330px
  text-align center
  color darkred
  i
    color darkred
</style>
