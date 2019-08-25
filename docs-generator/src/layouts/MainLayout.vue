<template>
  <q-layout view="hHh lpR fFr">
    <q-btn
      flat
      dense
      round
      @click="rightDrawerOpen = !rightDrawerOpen"
      aria-label="Menu"
      color="black"
      class="fixed-right"
      style="margin:15px 5px 0 0;z-index:10000000"
    >
      <q-icon name="menu" />
    </q-btn>
    <!--
    <q-header elevated reveal :reveal-offset="250">
      <q-toolbar>
        <q-btn
          flat
          dense
          round
          @click="leftDrawerOpen = !leftDrawerOpen"
          aria-label="Menu"
          color=""
        >
          <q-icon name="menu" />
        </q-btn>

        <q-toolbar-title>
          Tauri <span class="text-subtitle2">v{{ version }}</span>
        </q-toolbar-title>

        <div>Quasar v{{ $q.version }}</div>

        <q-btn
          flat
          dense
          round
          @click="rightDrawerOpen = !rightDrawerOpen"
          aria-label="Table of Contents"
        >
          <q-icon name="menu" />
        </q-btn>

      </q-toolbar>
    </q-header>
    -->

    <q-drawer
      v-model="rightDrawerOpen"
      side="right"
      bordered
      behavior="mobile"
      content-style="background-color: #f8f8ff;margin-top: 260px;padding-top:30px"
    >
      <q-scroll-area class="fit">
        <q-list dense>
          <q-item
            v-for="item in toc"
            :key="item.id"
            clickable
            v-ripple
            dense
            @click="scrollTo(item.id)"
            :active="activeToc === item.id"
          >
          <q-item-section v-if="item.level > 1" side> • </q-item-section>
            <q-item-section
              :class="`toc-item toc-level-${item.level}`"
            >{{ item.label }}</q-item-section>
          </q-item>
        </q-list>
      </q-scroll-area>
    </q-drawer>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script>
import { mapGetters } from 'vuex'
import { scroll } from 'quasar'

export default {
  name: 'MainLayout',

  data () {
    return {
      leftDrawerOpen: this.$q.platform.is.desktop,
      rightDrawerOpen: this.$q.platform.is.desktop,
      activeToc: 0
    }
  },

  computed: {
    ...mapGetters({
      toc: 'common/toc'
    })
  },

  mounted () {
    // code to handle anchor link on refresh/new page, etc
    if (location.hash !== '') {
      const id = location.hash.substring(1, location.hash.length)
      setTimeout(() => {
        this.scrollTo(id)
      }, 200)
    }
  },

  methods: {
    scrollTo (id) {
      this.activeToc = id
      const el = document.getElementById(id)

      if (el) {
        this.scrollPage(el)
      }
    },
    scrollPage (el) {
      const offset = el.offsetTop - 50
      scroll.setScrollPosition(window, offset, 500)
    }
  }
}
</script>

<style lang="stylus">
.toc-level
  &-2
    margin-left 0
  &-3
    margin-left 10px
</style>
