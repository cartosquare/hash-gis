"use client"

import { invoke } from './lib/tauri';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import ModelCard from './components/model-carad';
import { useModel } from './context/model-context';
import { ModelConfig } from './types';
import { message } from '@tauri-apps/api/dialog';

export default function Home() {
  const router = useRouter();
  const { model, setModel } = useModel();
  const [appConfig, setAppConfig] = useState<ModelConfig>();

  const navigate_dashboard = (modelIndex: number) => {
    if (appConfig?.models[modelIndex]) {
      appConfig.models[modelIndex].license_server = appConfig.license_server;
      setModel(appConfig.models[modelIndex]);
    }

    router.push('/dashboard');
  }

  useEffect(() => {
    invoke("app_config")
      .then((config) => {
        setAppConfig(config as ModelConfig);
      })
      .catch((msg) => {
        console.log("error: ", msg);
        message("读取配置文件失败！请检查app_config.toml文件。", {title: '读取配置文件失败', type: 'error'});
      });
  }, [])

  return (
    <main className="flex min-h-screen flex-col h-full items-center justify-between bg-base-300">
      <div className='flex w-full grow bg-base-300 p-4 justify-center'>
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          {
            appConfig && appConfig.models.map((model, index) => (

              <a key={index}
                className='hover:cursor-pointer hover:drop-shadow-lg'
                onClick={() => { navigate_dashboard(index) }}
              >
                <ModelCard
                  img={model.icon}
                  title={model.name}
                  description={model.description}
                  tags={model.tags}
                  isNew={true}
                />
              </a>
            ))
          }
        </div>
      </div>
    </main>
  )
}
