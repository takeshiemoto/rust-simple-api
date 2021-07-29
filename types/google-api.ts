export type IndustryIdentifier = {
  type: string;
  identifier: string;
};

export type ReadingModes = {
  text: boolean;
  image: boolean;
};

export type PanelizationSummary = {
  containsEpubBubbles: boolean;
  containsImageBubbles: boolean;
};

export type ImageLinks = {
  smallThumbnail: string;
  thumbnail: string;
};

export type ListPrice = {
  amount: number;
  currencyCode: string;
};

export type RetailPrice = {
  amount: number;
  currencyCode: string;
};

export type Epub = {
  isAvailable: boolean;
  acsTokenLink?: string;
};

export type Pdf = {
  isAvailable: boolean;
  acsTokenLink?: string;
};

export type SearchInfo = {
  textSnippet: string;
};

export type Offer = {
  finskyOfferType: number;
  listPrice: {
    amountInMicros: number;
    currencyCode: string;
  };
  retailPrice: {
    amountInMicros: number;
    currencyCode: string;
  };
};

export type AccessInfo = {
  country: string;
  viewability: string;
  embeddable: boolean;
  publicDomain: boolean;
  textToSpeechPermission: string;
  epub: Epub;
  pdf: Pdf;
  webReaderLink: string;
  accessViewStatus: string;
  quoteSharingAllowed: boolean;
};

export type VolumeInfo = {
  title: string;
  authors?: string[];
  publisher?: string;
  publishedDate: string;
  description?: string;
  industryIdentifiers: IndustryIdentifier[];
  readingModes: ReadingModes;
  pageCount: number;
  printType: string;
  categories?: string[];
  maturityRating: string;
  allowAnonLogging: boolean;
  contentVersion: string;
  panelizationSummary: PanelizationSummary;
  imageLinks?: ImageLinks;
  language: string;
  previewLink: string;
  infoLink: string;
  canonicalVolumeLink: string;
  subtitle?: string;
  averageRating?: number;
  ratingsCount?: number;
};

export type SaleInfo = {
  country: string;
  saleability: string;
  isEbook: boolean;
  listPrice?: ListPrice;
  retailPrice?: RetailPrice;
  buyLink?: string;
  offers?: Offer[];
};

export type Item = {
  kind: string;
  id: string;
  etag: string;
  selfLink: string;
  volumeInfo: VolumeInfo;
  saleInfo: SaleInfo;
  accessInfo: AccessInfo;
  searchInfo?: SearchInfo;
};

export type GoogleBookSearchResponse = {
  kind: string;
  totalItems: number;
  items: Item[];
};
