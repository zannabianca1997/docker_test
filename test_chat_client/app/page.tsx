import TestChat from "./TestChat";
import { env } from 'process'
import { unstable_noStore as noStore } from 'next/cache'

async function get_api_url(): Promise<URL> {
  noStore(); // do not prerender this page: we need the API url
  if (env.APIURL === undefined) {
    throw new Error("APIURL must be defined as a enviromental variable")
  }
  return new URL(env.APIURL);
}

export default async function Page() {
  const APIURL = await get_api_url();
  return <TestChat APIURL={APIURL.toString()} />;
}

