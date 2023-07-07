"use client"

import dynamic from 'next/dynamic';
import { useRouter } from 'next/navigation';
import { open, save } from "@tauri-apps/api/dialog"
import { useEffect, useState } from 'react';
import L, { latLngBounds } from 'leaflet';
import { invoke } from '../lib/tauri';
import { listen } from '@tauri-apps/api/event';
import { useModel } from '../context/model-context';
import { Body, fetch, Response } from '@tauri-apps/api/http';
import { MapSettings } from '../types';
// import Map from '../map';

const MapWithNoSSR = dynamic(() => import('../map'), {
  ssr: false,
});


interface PredictStatus {
  stage: string,
  progress: number,
  params: PredictParams,
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
  const { model, setModel } = useModel();
  const [inputFile, setInputFile] = useState<string>("");
  const [outputFile, setOutputFile] = useState<string>("");
  const [mapSettings, setMapSettings] = useState<MapSettings[]>([]);
  const [predictStatus, setPredictStatus] = useState<PredictStatus>();

  const navigatorHome = () => {
    router.push("/");
  }


  const createMapLayer = async (filepath: string, geoType: String) => {
    let bodyData: MapSettings = {
      name: "",
      path: filepath,
      xml: null,
      extent: null,
      geotransform: null,
      style: null,
      no_data_value: null,
      spatial_ref_code: null,
      spatial_units: null,
      driver_name: null,
      bounds: null
    };

    if (geoType == "vector") {
      const style = `
<Map srs="epsg:3857">
	<Style name="My Style">
		<Rule>
			<PolygonSymbolizer fill="red" fill-opacity="1"/>
			<LineSymbolizer stroke="blue" stroke-opacity="1" stroke-width="0.1"/>
		</Rule>
	</Style>
	<Layer name="" srs="epsg:4326">
		<StyleName>My Style</StyleName>
		<Datasource>
			<Parameter name="file">${filepath}</Parameter>
			<Parameter name="layer_by_index">0</Parameter>
			<Parameter name="type">ogr</Parameter>
		</Datasource>
	</Layer>
</Map>
    `
      bodyData.xml = style;

    } else {
      bodyData.style = {
          colours: [
            [0, 0, 0], [255, 255, 255]
          ],
          bands: [1, 2, 3],
          name: null,
          vmin: null,
          vmax: null,
        };
    }

    try {
      const rawResponse = await fetch<MapSettings>(`http://localhost:8080/map`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: Body.json(bodyData),
      });
      if (rawResponse.status != 200) {
        console.log('error', rawResponse);
      } else {
        const response = rawResponse.data;
        if (response.bounds) {
          // console.log(response);
          setMapSettings([...mapSettings, response]);
        }
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
    setMapSettings([]);
    setOutputFile("");

    // create map view
    await createMapLayer(file as string, "raster");
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
      const unlistenPredict = await listen<PredictStatus>('predict-status', async (event) => {
        console.log('receive event', event.payload);
        setPredictStatus(event.payload as PredictStatus);
      });
      return unlistenPredict;
    }

    const unlistenPredict = createListenEvent();
    return () => {
    }
  }, [])

  useEffect(() => {
    if (!predictStatus) {
      return
    }

    if (predictStatus.stage == "结束" && predictStatus.progress != -1) {
      createMapLayer(predictStatus.params.outputPath, "vector");
    }
  }, [predictStatus])

  return (
    <main className="flex min-h-screen flex-col h-full items-center justify-between bg-base-300">
      <div className='flex flex-row w-full grow bg-base-300 p-4'>

        <div className='flex flex-col gap-4'>
          <button className='btn w-32' onClick={navigatorHome}>

            <svg viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" width="19" height="19"><path d="M8 1L1 7.5 8 14m5.5-13l-7 6.5 7 6.5" stroke="currentColor" strokeLinecap="square"></path></svg>
            返回
          </button>

          {/* 必选参数 */}
          <div tabIndex={0} className="collapse collapse-open bg-base-200">
            <input type="checkbox" />
            <div className="collapse-title text-lg">
              必选参数
            </div>

            <div className="flex flex-col collapse-content">

              {/* 批处理选项 */}
              <div className="form-control">
                <label className="label cursor-pointer">
                  <span className="label-text">批处理</span>
                  <input type="checkbox" className="toggle toggle-primary" />
                </label>
              </div>
              {/* 输入 */}
              {
                model && Array.from(Array(model.input_files).keys()).map((val) => (
                  <div className='form-control' key={val}>
                    <label className='label cursor-pointer'>
                      <span className="label-text w-24">输入 {model.input_files > 1 ? val + 1 : ''}</span>
                      <div className='join'>
                        <input type="text" value={inputFile} className="input join-item" readOnly />
                        <button onClick={openInputFile} className='btn btn-neutral join-item'>选择..</button>
                      </div>
                    </label>
                  </div>
                ))
              }

              {/* 输出 */}
              <div className='form-control'>
                <label className='label cursor-pointer'>
                  <span className="label-text w-24">输出</span>
                  <div className='join'>
                    <input type="text" value={outputFile} className="input join-item" readOnly />
                    <button onClick={saveOutputFile} className='btn btn-neutral join-item'>选择..</button>
                  </div>

                </label>
              </div>

              {/* 开启GPU */}
              <div className="form-control">
                <label className="label cursor-pointer">
                  <span className="label-text">开启GPU</span>
                  <input type="checkbox" className="toggle toggle-primary" defaultChecked />
                </label>
              </div>

              {/* 选择GPU */}
              <div className="form-control">
                <label className="label cursor-pointer">
                  <span className="label-text">GPU</span>
                  <select defaultValue="0" className="select select-bordered w-full max-w-xs">
                    <option value="0">Nvidia 1080Ti</option>
                    <option value="1">Nvidia V100</option>
                    <option value="2">Nvidia T4</option>
                  </select>
                </label>
              </div>

            </div>
          </div>

          {/* 高级参数 */}
          <div tabIndex={1} className="collapse bg-base-200">
            <input type="checkbox" />
            {/* <div className="collapse-title text-xl font-medium"> */}
            <div className="collapse-title text-lg">
              高级参数
            </div>

            <div className="flex flex-col collapse-content">
              {
                model && model.options.map((option, index) => (
                  <div key={index} className="form-control">
                    <label className="label cursor-pointer">
                      <span className="label-text w-24">{option.label}</span>
                      {
                        option.input_type == "range" &&
                        <input type="range" min={option.min} max={option.max} defaultValue={option.value} className={`range ${option.style}`} />
                      }
                      {
                        option.input_type == "select" &&
                        <select defaultValue="0" className={`select select-bordered w-full max-w-xs ${option.style}`}>
                          {
                            option.choices && option.choices.map((choice, index) => (
                              // index == 0 ? <option selected key={index}>{choice}</option> : <option key={index}>{choice}</option>
                              <option key={index} value={index}>{choice}</option>
                            )
                            )
                          }
                        </select>
                      }
                      {
                        option.input_type == "text" &&
                        <input type="text" placeholder="" className="input w-full max-w-xs" />
                      }
                    </label>
                  </div>
                ))
              }

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

        <MapWithNoSSR
          settings={mapSettings}
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
