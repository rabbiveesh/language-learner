<template>
  <img :class="['col', { selected: selected }]" :src="src"
    @click="select" />
</template>

<script>
export default {
  name: 'SelectableImg',
  props: [ 'src', 'selected' ],
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
