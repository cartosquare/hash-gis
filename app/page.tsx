"use client"

import dynamic from 'next/dynamic';
import LeftMenuManager from './left-menu-manager'

const MapWithNoSSR = dynamic(() => import('./map'), {
    ssr: false,
});


export default function Home() {
  return (
    <main className="flex min-h-screen flex-col h-full items-center justify-between bg-base-300">
      <div className='flex w-full grow bg-base-300'>
        <LeftMenuManager />
        <MapWithNoSSR />
      </div>
    </main>
  )
}
