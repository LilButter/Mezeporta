<script setup>
import { assetUrl } from "../store";

const gearUrl = assetUrl("/extra/gear.png");
</script>

<template>
  <img
    :src="gearUrl"
    aria-hidden="true"
    class="gear-icon-image"
    alt=""
    draggable="false"
  />
</template>
