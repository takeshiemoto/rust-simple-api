<template>
  <CThemeProvider>
    <CColorModeProvider>
      <CBox font-family="body" as="main">
        <CReset />
        <Nuxt />
      </CBox>
    </CColorModeProvider>
  </CThemeProvider>
</template>
<script lang="ts">
import {
  CThemeProvider,
  CColorModeProvider,
  CReset,
  CBox,
} from '@chakra-ui/vue';
import { defineComponent } from '@nuxtjs/composition-api';

export default defineComponent({
  name: 'App',
  components: {
    CThemeProvider,
    CColorModeProvider,
    CReset,
    CBox,
  },
});
</script>
