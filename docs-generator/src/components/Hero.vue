<template>
  <div class="full-width">
    <q-page-sticky expand class="page-header fixed-top shadow-8 scroll-determined" v-scroll="scrolled">
      <q-chip outline dense square icon="star" icon-right="star" class="claim absolute-center text-weight-light bg-amber-3" style="margin-top:135px">Build highly secure native apps that have tiny binaries and are very fast.</q-chip>
      <div class="bg-container scroll-determined q-pa-md q-ml-lg"></div>
      <div>
        <div>
          <img src="statics/tauri-text.png" class="tauri-name scroll-determined">
        </div>

        <div v-if="buttons" style="margin-top:100px">
          <q-btn dense size="small" type="a" href="https://github.com/quasarframework/tauri" target="_blank" class="btn" label="Quick Start" no-caps color="warning" text-color="black"/>
          <q-btn dense size="small" to="/docs/patterns" class="btn" label="Patterns" no-caps  color="warning" text-color="black"/>
          <q-btn dense size="small" to="/docs/environment" class="btn" label="Environment" no-caps color="warning" text-color="black" />
        </div>
      <div v-else class="absolute-right q-pa-lg q-mt-xs" >
        <q-btn-dropdown color="warning" label="Quick Links" no-caps text-color="black" class="q-mr-lg">
          <q-list color="warning">
            <q-item clickable v-close-popup type="a" href="https://github.com/quasarframework/tauri" target="_blank">
              <q-item-section>
                <q-item-label>Quick Start</q-item-label>
              </q-item-section>
            </q-item>

            <q-item clickable v-close-popup>
              <q-item-section>
                <q-item-label>Patterns</q-item-label>
              </q-item-section>
            </q-item>

            <q-item clickable v-close-popup>
              <q-item-section>
                <q-item-label>Environment</q-item-label>
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
  methods: {
    scrolled (position) {
      this.height = 270 - (position / 4)
      this.heightName = 140 - (position / 4)
      this.heightPic = 250 - (position / 4)
      if (this.height <= 90) {
        this.height = 90
        this.heightPic = 90
        this.buttons = false
      }
      if (this.height <= 200) {
        this.buttons = false
      } else {
        this.buttons = true
      }
      if (this.heightName <= 50) {
        this.heightName = 50
      }
      // todo: cleanup, use vuex
      document.getElementsByClassName('q-drawer__content')[0].setAttribute('style', `margin-top: ${this.height + 10}px`)
      document.getElementsByClassName('claim')[0].setAttribute('style', `margin-top: ${(this.height / 2) + 3}px`)
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
.q-menu
  z-index 1000000
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
  min-height 90px
  min-width 90px
</style>
