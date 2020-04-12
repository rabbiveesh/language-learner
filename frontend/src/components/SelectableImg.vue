<template>
  <div class="column inline-block">
    <img :class="{ selected: selected, 'q-ma-md': selected }" :src="src"
      @click="select" />
    <span v-if="selected" v-html="text" />
  </div>
</template>

<script>
export default {
  name: 'SelectableImg',
  props: [ 'src', 'selected', 'text' ],
  data () {
    return {
    }
  },
  methods: {
    select () {
      this.$emit('selected', !this.selected)
    }
  }
}
</script>

<style>
  .selected {
    transform: scale(1.25);
    border: 1px solid blue;
    box-shadow: 0px 0px 10px 7.5px blue;
  }
</style>
