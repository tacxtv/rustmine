<template lang="pug">
  div.fit.overflow-hidden(:style="{ background: `url(${backgroundUrl}) no-repeat center center fixed` }")
    q-splitter.full-height.gradient(v-model="LayoutSplitter" disable)
      template(#before)
        q-splitter(v-model="menuSplitter" horizontal disable)
          template(#before)
            q-tabs.overflow-hidden(
              v-model="tab"
              vertical outside-arrows
            )
              q-tab(
                v-for="i in 15" :key="i" :label="`Tab ${i}`"
                :name="'tab-' + i"
                icon="mdi-home"
              )
          template(#after)
            account-avatar
      template(#after)
        div.fit.flex(:style="{ 'flex-flow': 'column' }")
          q-tab-panels.fit.transparent(
            v-model="tab"
            transition-prev="jump-up"
            transition-next="jump-down"
            animated
          )
            q-tab-panel.fit(v-for="i in 15" :key="i" :name="'tab-' + i")
              div.fit.text-center
                div.text-h3 MODPACK
          q-bar
            q-space
            small 0.0.1
</template>

<script setup lang="ts">
const tab = ref(null)
const menuFlow = ref('column')
const LayoutSplitter = ref(25)
const menuSplitter = ref(70)
const backgroundUrl = ref("https://i.pinimg.com/originals/42/48/dc/4248dc2842854731abc80bee7ecb1346.png")
</script>
