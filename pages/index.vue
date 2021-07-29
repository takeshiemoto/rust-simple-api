<template>
  <CBox p="6">
    <form @submit.prevent="submit">
      <CFlex grid-gap="2">
        <CInput v-model="keyword" placeholder="キーワードを入力" />
        <CButton type="submit" variant-color="blue">検索</CButton>
        <CButton @click="clear">クリア</CButton>
      </CFlex>
    </form>
    <CBox py="4">
      <CSimpleGrid :columns="4" :spacing="10">
        <CBox v-for="item in items" :key="item.id">
          <CStack>
            <CImage
              v-if="item.volumeInfo.imageLinks"
              :src="item.volumeInfo.imageLinks.thumbnail"
              size="200px"
              object-fit="cover"
            />
            <CText>{{ item.volumeInfo.title }}</CText>
          </CStack>
        </CBox>
      </CSimpleGrid>
    </CBox>
  </CBox>
</template>

<script lang="ts">
import { defineComponent, reactive, toRefs } from '@nuxtjs/composition-api';
import {
  CBox,
  CInput,
  CButton,
  CFlex,
  CImage,
  CSimpleGrid,
  CText,
  CStack,
} from '@chakra-ui/vue';
import { GoogleBookSearchResponse, Item } from '~/types/google-api';

type FormType = {
  keyword: string;
};

export default defineComponent({
  components: {
    CBox,
    CInput,
    CButton,
    CFlex,
    CSimpleGrid,
    CText,
    CImage,
    CStack,
  },
  setup() {
    const formState = reactive<FormType>({
      keyword: '',
    });

    const results = reactive<{ items: Item[] }>({
      items: [],
    });

    const searchBook = async (keyword: string) => {
      const res = await fetch(
        `https://www.googleapis.com/books/v1/volumes?q=${keyword}`
      ).then((res) => res.json());

      return res as GoogleBookSearchResponse;
    };

    const clear = () => {
      formState.keyword = '';
      results.items = [];
    };

    const submit = async () => {
      const response = await searchBook(formState.keyword);
      results.items = response.items;
    };

    return {
      ...toRefs(formState),
      ...toRefs(results),
      submit,
      clear,
    };
  },
});
</script>
