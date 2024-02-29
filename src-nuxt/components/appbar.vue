<template lang="pug">
  div
    q-bar.bg-red
      q-toolbar-title RustMine
      q-space
      q-btn.icon-only(@click="minimize" dense flat)
        q-icon(name="mdi-window-minimize")
      q-btn.icon-only(dense flat)
        q-icon(name="mdi-window-restore")
      q-btn.icon-only(dense flat)
        q-icon(name="mdi-window-maximize")
      q-btn.icon-only(dense flat)
        q-icon(name="mdi-window-close")
</template>

<script setup lang="ts">
import {appWindow} from "@tauri-apps/api/window";

async function minimize() {
  await appWindow.minimize()
}
// invoke("get_title").then((title: string) => {
//   console.log(title);
// });
</script>
