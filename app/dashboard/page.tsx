"use client"

// import dynamic from 'next/dynamic';
import Map from '../map';
import { useRouter } from 'next/navigation';
import { open, save } from "@tauri-apps/api/dialog"
import { useEffect, useState } from 'react';
import L, { latLngBounds } from 'leaflet';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

// const MapWithNoSSR = dynamic(() => import('../map'), {
//   ssr: false,
// });


interface PredictStatus {
  stage: string,
  progress: number,
}

type PredictParams = {
  algorithmType: string,
  modelPath: string,
  datasources: string[],
  options: string[],
  outputPath: string,
}

export default function Home() {
  const router = useRouter();
  const [inputFile, setInputFile] = useState<string>("");
  const [outputFile, setOutputFile] = useState<string>("");
  const [layers, setLayers] = useState<string[]>([]);
  const [bounds, setBounds] = useState<L.LatLngBounds>();
  const [predictStatus, setPredictStatus] = useState<PredictStatus>();

  const navigatorHome = () => {
    router.push("/");
  }

  const createMapLayer = async (filepath: string) => {
    try {
      const rawResponse = await fetch(`http://localhost:8080/map`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          'name': "",
          "path": filepath,
          "style": {
            "colours": [
              [0, 0, 0], [255, 255, 255]
            ],
            "bands": [1, 2, 3]
          }
        })
      });
      if (rawResponse.status != 200) {
        //toast.error(`请求失败：${rawResponse.status}`);
        console.log('error');
      } else {
        const response = (await rawResponse.json());
        console.log(response)
        setLayers([`http://localhost:8080/${response.name}/{z}/{x}/{y}.png`]);
        const b = new L.LatLngBounds(L.latLng(response.bounds[0], response.bounds[1]), L.latLng(response.bounds[2], response.bounds[3]))
        console.log(b)
        setBounds(b
        );
      }
    } catch (error) {
      console.log(error);
      // toast.error(`请求失败：${error}`);
      console.log('error!', error);
    }
  }

  const openInputFile = async () => {
    const file = await open();
    setInputFile(file as string);

    // create map view
    await createMapLayer(file as string);
  }

  const saveOutputFile = async () => {
    const file = await save();
    setOutputFile(file as string);
  }

  const createPredictTask = () => {
    invoke('predict', {
      params: {
        algorithmType: "seg-post",
        modelPath: "D:\\atlas\\model\\sense-layers\\agri\\corn_rgbnir8bit_2m_221223.m",
        datasources: [inputFile],
        options: ["license_server=10.112.60.244:8181", "verbose=debug"],
        outputPath: outputFile,
      }
    });
  }

  useEffect(() => {
    const createListenEvent = async () => {
      const unlistenPredict = await listen<PredictStatus>('predict-status', (event) => {
        console.log('receive event', event.payload);
        setPredictStatus(event.payload as PredictStatus);
      });
      return unlistenPredict;
    }

    const unlistenPredict = createListenEvent();
    return () => {
    }
  }, [])

  return (
    <main className="flex min-h-screen flex-col h-full items-center justify-between bg-base-300">
      <div className='flex flex-row w-full grow bg-base-300 p-4'>

        <div className='flex flex-col gap-4'>
          <button className='btn w-32' onClick={navigatorHome}>

            <svg viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" width="19" height="19"><path d="M8 1L1 7.5 8 14m5.5-13l-7 6.5 7 6.5" stroke="currentColor" strokeLinecap="square"></path></svg>
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
                  <div className='join'>
                    <input type="text" value={inputFile} className="input join-item" readOnly />
                    <button onClick={openInputFile} className='btn btn-neutral join-item'>选择..</button>
                  </div>
                </label>
              </div>

              <div className='form-control'>
                <label className='label cursor-pointer'>
                  <span className="label-text w-24">输出</span>
                  <div className='join'>
                    <input type="text" value={outputFile} className="input join-item" readOnly />
                    <button onClick={saveOutputFile} className='btn btn-neutral join-item'>选择..</button>
                  </div>

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

          <div className='flex justify-end pr-6'>
            <button onClick={createPredictTask} className='btn btn-primary w-32'>启动</button>
          </div>

        </div>


        {/* <MapWithNoSSR /> */}
        <Map
          layers={layers}
          bounds={bounds}
        />
      </div>
      {
        predictStatus &&
        <footer className="footer footer-center p-4 bg-base-300 text-base-content">
          <div className='w-full'>
            <div className='flex flex-row w-full justify-start'>

              <div className='w-32'>
                <p className="">{predictStatus.stage}</p>
              </div>
              <div className='w-full'>
                <progress className="progress progress-primary" value={predictStatus.progress * 100} max="100"></progress>
              </div>
            </div>

          </div>
        </footer>
      }
    </main>
  )
}
