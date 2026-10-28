// src/i18n/locales/fr/index.ts
// Typed against English: a missing key in any fr/*.json is a compile error here.
import type en from '../en'
import auth from './auth.json'
import errors from './errors.json'
import validation from './validation.json'
 
const fr: typeof en = { auth, errors, validation }
 
export default fr
 