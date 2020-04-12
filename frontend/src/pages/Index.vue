<template>
  <q-page class="flex flex-center column justify-between">
    <q-tabs class="col-1 row" v-model="tab" align="center" >
      <q-tab v-for="word in words" :key="word" :label="word"
        :name="word"/>
    </q-tabs>

    <q-tab-panels  class="col-11" v-model="tab" animated >
      <!-- I think it should be fit, but fill seems to work for
      some reason -->
      <q-tab-panel class="fill" :name="word" v-for="word in words"
        :key="word" >
        <images-tab :word="word"/>
      </q-tab-panel>
    </q-tab-panels>

    <q-page-sticky position="bottom-right" :offset="[18,18]" >
      <q-btn color="primary" fab icon="keyboard_arrow_up" @click="getWords" />
    </q-page-sticky>
  </q-page>
</template>

<script>
import ImagesTab from '../components/ImagesTab.vue'

export default {
  name: 'PageIndex',
  components: { ImagesTab },
  data () {
    return {
      words: ['your', 'face'],
      tab: ''
    }
  },
  methods: {
    getWords () {
      this.$axios.get('/words')
        .then((response) => {
          this.words = response.data
          this.tab = this.words[0]
        })
    }
  },
  mounted () {
    this.getWords()
    window.addEventListener('unload',
      () => {
        navigator.sendBeacon(`${this.$baseURL}/exit`)
      }
    )
  }
}
</script>
