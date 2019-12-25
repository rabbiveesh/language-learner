<template>
  <q-page class="flex flex-center column">
    <q-tabs v-model="tab" >
      <q-tab v-for="word in words" :key="word" :label="word"
        :name="word"/>
    </q-tabs>

    <q-tab-panels v-model="tab" animated >
      <q-tab-panel class="fill" :name="word" v-for="word in words"
        :key="word" >
        <images-tab :word="word"/>
      </q-tab-panel>
    </q-tab-panels>

    <q-page-sticky position="bottom-right" :offset="[18,18]" >
      <q-btn fab icon="refresh" @click="getWords" />
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
      this.$axios.get('http://localhost:9099/words')
        .then((response) => {
          this.words = response.data
          this.tab = this.words[0]
        })
    }
  },
  mounted () {
    this.getWords()
    window.addEventListener('unload',
      () => navigator.sendBeacon('/exit')
    )
  }
}
</script>
