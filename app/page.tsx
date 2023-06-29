"use client"

import { useRouter } from 'next/navigation';
import ModelCard from './components/model-carad';

export default function Home() {
  const router = useRouter();

  const navigate_dashboard = () => {
    router.push('/dashboard');
  }

  return (
    <main className="flex min-h-screen flex-col h-full items-center justify-between bg-base-300">
      <div className='flex w-full grow bg-base-300 p-4'>
        <div className="grid grid-cols-4 gap-4">
          <a
            className='hover:cursor-pointer hover:drop-shadow-lg'
            onClick={navigate_dashboard}
          >
            <ModelCard
              img='./building.png'
              title='建筑提取'
              description='建筑物检测产品基于商汤自研深度学习遥感解译算法，单景或批量输入最佳分辨率为0.3米或2米的8bit、RGB、三波段的遥感影像，自动进行shp格式建筑物矢量轮廓提取。'
              tags={["3波段", "亚米影像"]}
              isNew={true}
            />
          </a>
        </div>
      </div>
    </main>
  )
}
