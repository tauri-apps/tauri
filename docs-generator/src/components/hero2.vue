<template>
<q-page-sticky expand position="top" :offset="[0,0]" class="p-hero-container">
    <div class="p-hero" :style="styleHero">
        <div class="p-hero-title ellipsis non-selectable" :style="styleTitle">
            <slot name="title"></slot>
        </div>
        <div class="row q-gutter-x-md justify-between">
            <div class="p-hero-subtitle col ellipsis non-selectable">
                <slot name="subtext"></slot>
            </div>
            <div class="col-auto">
                <slot name="action"></slot>
            </div>
        </div>
    </div>
</q-page-sticky>
</template>

<script>
import { mapState } from "vuex"
export default {
    name: "PHero",
    props: {
        backgroundImage: {
            type: String,
            default: "default-logo"
        }
    },
    computed: {
        ...mapState({
            frameless: state => state.app.window.frameless,
            y: state => state.app.scroll.current.y,
        }),
        minHeight() {
            return this.frameless ? 68 : 100
        },
        maxHeight() {
            return this.frameless ? 150 : 180
        },
        distance() {
            return this.maxHeight - this.minHeight
        },
        percent() {
            return 1 - Math.min(1, this.y / this.distance)
        },
        styleHero() {
            return {
                height: `${this.minHeight + this.percent * this.distance}px`,
                backgroundImage: `url("/statics/hero/${this.backgroundImage}.svg")`,
                backgroundSize: `auto ${200 - 50 * this.percent}%`
            }
        },
        styleTitle() {
            return {
                transform: `scale(${1 + this.percent * 0.625})`,
                //fontSize: `${16 + this.percent * 12}px`,
                //marginTop: `${0 + this.percent * 20}px`,
                //marginBottom: `${0 + this.percent * 20}px`,
                marginTop: `${5 + this.percent * 20}px`,
                marginBottom: `${0 + this.percent * 30}px`,
                //height: `${45 * this.percent}px`,
            }
        }
    }
}
</script>

<style>
</style>
