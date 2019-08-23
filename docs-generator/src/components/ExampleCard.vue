<template>
  <section :id="slugifiedTitle" class="q-pa-md overflow-auto">
    <q-card flat bordered class="no-shadow">
      <q-toolbar>
        <q-ribbon
          position="left"
          color="rgba(0,0,0,.58)"
          background-color="#c0c0c0"
          leaf-color="#a0a0a0"
          leaf-position="bottom"
          decoration="rounded-out"
        >
          <q-toolbar-title
          class="example-title"
          style="padding: 5px 20px;"
          @click="copyHeading(slugifiedTitle)"><span class="ellipsis">{{ title }}</span></q-toolbar-title>
        </q-ribbon>
      </q-toolbar>
      <q-separator v-if="this.$slots.default" />
      <q-card-section v-if="this.$slots.default">
        <slot></slot>
      </q-card-section>
      <q-separator />
      <q-expansion-item
        group="someGroup"
        caption="Code"
      >
        <q-card>
          <q-tabs
            v-model="tab"
            dense
            class="text-grey"
            active-color="primary"
            indicator-color="primary"
            align="left"
            narrow-indicator
          >
            <q-tab name="template" v-if="parts.template" label="Template" />
            <q-tab name="script" v-if="parts.script" label="Script" />
            <q-tab name="style" v-if="parts.style" label="Style" />
          </q-tabs>

          <q-separator />

          <q-tab-panels v-model="tab" animated>
            <q-tab-panel v-if="parts.template" name="template">
              <q-markdown :src="parts.template" />
            </q-tab-panel>

            <q-tab-panel v-if="parts.script" name="script">
              <q-markdown :src="parts.script" />
            </q-tab-panel>

            <q-tab-panel v-if="parts.style" name="style">
              <q-markdown :src="parts.style" />
            </q-tab-panel>

          </q-tab-panels>
        </q-card>
      </q-expansion-item>
      <q-separator />

      <component v-bind:is="name" style="overflow: hidden;" />

    </q-card>
  </section>
</template>

<script>
import { copyHeading, slugify } from 'assets/page-utils'

export default {
  name: 'ExampleCard',

  components: {
    TauriBasic: () => import('../examples/TauriBasic'),
    TauriAdvanced: () => import('../examples/TauriAdvanced')
  },

  props: {
    title: {
      type: String,
      required: true
    },
    name: {
      type: String,
      required: true
    },
    tagParts: {
      type: Object,
      default: () => {}
    }
  },

  data () {
    return {
      tab: 'template',
      parts: {}
    }
  },

  mounted () {
    this.updateParts()
  },

  computed: {
    slugifiedTitle () {
      return slugify(this.title)
    }
  },

  methods: {
    copyHeading,
    updateParts () {
      this.parts = {}
      Object.keys(this.tagParts).forEach(key => {
        if (this.tagParts[key] !== '') {
          this.parts[key] = '```\n' + this.tagParts[key] + '\n```'
        }
      })
    }
  }
}
</script>
