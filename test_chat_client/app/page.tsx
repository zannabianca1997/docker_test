import TestChat from "./TestChat";
import { env } from 'process'
import { unstable_noStore as noStore } from 'next/cache'

async function get_api_url(): Promise<URL> {
  noStore(); // do not prerender this page: we need the API url
  if (env.API_URL === undefined) {
    throw new Error("API_URL must be defined as a enviromental variable")
  }
  return new URL(env.API_URL);
}

export default async function Page() {
  const API_URL = await get_api_url();
  return <TestChat API_URL={API_URL.toString()} />;
}

