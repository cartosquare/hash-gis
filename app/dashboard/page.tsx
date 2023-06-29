"use client"

import dynamic from 'next/dynamic';
import { useRouter } from 'next/navigation';

const MapWithNoSSR = dynamic(() => import('../map'), {
  ssr: false,
});

export default function Home() {
  const router = useRouter();

  const navigator_home = () => {
    router.push("/");
  }

  return (
    <main className="flex min-h-screen flex-col h-full items-center justify-between bg-base-300">
      <div className='flex flex-row w-full grow bg-base-300 p-4'>

        <div className='flex flex-col gap-4'>
          <button className='btn w-32' onClick={navigator_home}>

            <svg viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" width="19" height="19"><path d="M8 1L1 7.5 8 14m5.5-13l-7 6.5 7 6.5" stroke="currentColor" stroke-linecap="square"></path></svg>
            返回
          </button>
          <div tabIndex={0} className="collapse collapse-open bg-base-200">
            <input type="checkbox" />
            <div className="collapse-title text-lg">
              必选参数
            </div>

            <div className="flex flex-col collapse-content">

              <div className='form-control'>
                <label className='label cursor-pointer'>
                  <span className="label-text w-24">输入</span>
                  <input type="file" className="file-input file-input-sm file-input-bordered w-full max-w-xs" />
                </label>
              </div>

              <div className='form-control'>
                <label className='label cursor-pointer'>
                  <span className="label-text w-24">输出</span>
                  <input type="file" className="file-input file-input-sm file-input-bordered w-full max-w-xs" />
                </label>
              </div>


              <div className="form-control">
                <label className="label cursor-pointer">
                  <span className="label-text">开启GPU</span>
                  <input type="checkbox" className="toggle toggle-primary" defaultChecked />
                </label>
              </div>

              <div className="form-control">
                <label className="label cursor-pointer">
                  <span className="label-text">GPU</span>
                  <select className="select select-bordered w-full max-w-xs">
                    <option disabled selected>Nvidia 1080Ti</option>
                    <option>Nvidia V100</option>
                    <option>Nvidia T4</option>
                  </select>
                </label>
              </div>

            </div>
          </div>
          <div tabIndex={1} className="collapse bg-base-200">
            <input type="checkbox" />
            {/* <div className="collapse-title text-xl font-medium"> */}
            <div className="collapse-title text-lg">
              高级参数
            </div>

            <div className="flex flex-col collapse-content">
              <div className="form-control">
                <label className="label cursor-pointer">
                  <span className="label-text w-24">概率阈值</span>
                  <input type="range" min={0} max={100} defaultValue={50} className="range" />
                </label>
              </div>

              <div className="form-control">
                <label className="label cursor-pointer">
                  <span className="label-text w-24">其它参数</span>
                  <input type="text" placeholder="-v --remove_small 100" className="input w-full max-w-xs" />
                </label>
              </div>
            </div>

          </div>

        </div>


        <MapWithNoSSR />
      </div>
    </main>
  )
}
