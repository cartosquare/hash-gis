"use client"

import dynamic from 'next/dynamic';
import { useRouter } from 'next/navigation';
import { open, save, message } from "@tauri-apps/api/dialog"
import { useEffect, useState } from 'react';
import { invoke } from '../lib/tauri';
import { listen } from '@tauri-apps/api/event';
import { useModel } from '../context/model-context';
// import MapSquare from '../map';
import { useMapLayers } from '../context/maplayers-context';
import { ModelOption } from '../types';
import { info, debug, warn, error } from "tauri-plugin-log-api";

const MapWithNoSSR = dynamic(() => import('../map'), {
  ssr: false,
});

interface PredictStatus {
  stage: string,
  progress: number,
  fail:  boolean,
  params: PredictParams,
}

type PredictParams = {
  algorithmType: string,
  modelPath: string,
  datasources: string[],
  options: string[],
  outputPath: string,
}

const max_text_length = 25;

type PredictOptions = Map<string, string | number | null>;

export default function Home() {
  const router = useRouter();
  const { model, setModel } = useModel();
  const mapLayers = useMapLayers();
  const [batchMode, setBatchMode] = useState<boolean>(false);
  const [inputFile, setInputFile] = useState<string[]>(Array(model?.input_files).fill(''));
  const [outputFile, setOutputFile] = useState<string>('');
  const [devideId, setDeviceId] = useState<number>(0);
  const [gpuList, setGpuList] = useState<string[]>([]);
  const [predictStatus, setPredictStatus] = useState<PredictStatus | null>(null);
  const [predicting, setPredicting] = useState<boolean>(false);
  const [predictOptions, setPredictOptions] = useState<PredictOptions>(new Map());
  const [predictMsg, setPredictMsg] = useState<string>("");

  const navigatorHome = () => {
    mapLayers.clearLayers();
    router.push("/");
  }

  const setPredictOption = (option: ModelOption, name: string, val: string | number) => {
    if (option.input_type == "range") {
      const value = parseFloat(val as string);
      predictOptions.set(option.name, option.scale ? value * option.scale : value);
    } else if (option.input_type == "select" && option.choices) {
      const value = parseInt(val as string);
      predictOptions.set(option.name, option.choices[value]);
    } else if (option.input_type == "text") {
      predictOptions.set(option.name, val);
    } else {
      message(`Invalid input type: ${option.input_type}`, { title: 'Error', type: 'error' });
    }
    debug(`update: ${JSON.stringify(predictOptions)}`);
    setPredictOptions(predictOptions);
  }

  const parsePredictionParameter = (): PredictParams => {
    if (!model || !outputFile) {
      return {
        algorithmType: "",
        modelPath: "",
        datasources: [],
        options: [],
        outputPath: "",
      }
    }

    const opt_device = `device=${devideId > 0 ? "cuda" : "cpu"}`;
    const opt_device_id = `device_id=${devideId > 0 ? devideId - 1 : devideId}`;
    let options: string[] = [opt_device, opt_device_id];

    if (model.license_server) {
      options.push(`license_server=${model.license_server}`);
    }

    model.options.forEach((option) => {
      const opt = `${option.name}=${predictOptions.get(option.name)}`;
      options.push(opt);
    });

    return {
      algorithmType: model.post_type,
      modelPath: model.model_path,
      datasources: inputFile,
      options,
      outputPath: outputFile,
    }
  }

  const onBatchModeChanged = () => {
    setBatchMode(!batchMode);
  }

  const openInputFile = async (index: number) => {
    if (!model) {
      return;
    }

    const file = await open({
      directory: batchMode,
      multiple: false,
      filters: [{
        name: 'GeoTIFF',
        extensions: ['tif', 'tiff']
      },
      {
        name: 'IMG',
        extensions: ['img', 'IMG']
      }
      ]
    });

    // user canceled
    if (file == null) {
      return;
    }

    if (inputFile.length > index) {
      let oldInputs = inputFile;
      oldInputs[index] = file as string;
      setInputFile(oldInputs);
    } else {
      let inputs = new Array<string>(model?.input_files);
      inputs[index] = file as string;
      setInputFile(inputs);
    }

    // setMapSettings([]);
    // setOutputFile("");

    // create map view
    if (!batchMode) {
      mapLayers.createLayer(file as string, 'raster');
    }
  }

  const saveOutputFile = async () => {
    if (!batchMode) {
      const file = await save({
        filters: [{
          name: 'Shapefile',
          extensions: ['shp']
        },
        {
          name: 'GeoJSON',
          extensions: ['geojson']
        }
        ]
      });
      setOutputFile(file as string);
      // mapLayers.createLayer(file as string, 'vector');
    } else {
      const file = await open({
        directory: batchMode,
        multiple: false
      });
      setOutputFile(file as string);
    }
  }

  const createPredictTask = () => {
    // if mapLayers.data.layers.length == 0)
    setPredicting(true);

    const params = parsePredictionParameter();
    info(JSON.stringify(params));
    invoke('predict', {
      // params: {
      //   algorithmType: "seg-post",
      //   modelPath: "D:\\atlas\\model\\sense-layers\\agri\\corn_rgbnir8bit_2m_221223.m",
      //   datasources: [inputFile],
      //   options: ["license_server=10.112.60.244:8181", "verbose=debug"],
      //   outputPath: outputFile,
      // }
      params,
    });
  }

  useEffect(() => {
    // 监听解译状态
    const createListenEvent = async () => {
      const unlistenPredict = await listen<PredictStatus>('predict-status', async (event) => {
        setPredictStatus(event.payload as PredictStatus);
      });
      return unlistenPredict;
    }
    const unlistenPredict = createListenEvent();

    // gpu list
    invoke<string[]>("get_cuda_info")
      .then((gpuList) => {
        setGpuList(gpuList);
      })
      .catch((msg) => {
        error("read GPU info fail!");
        message("读取可用GPU失败！请联系技术支持人员。", { title: '读取可用GPU失败', type: 'error' });
      })
    return () => {
    }
  }, [])

  // 初始解译参数
  useEffect(() => {
    if (!model) {
      return;
    }
    model.options.forEach((option) => {
      if (option.input_type == "range" && option.value) {
        predictOptions.set(option.name, option.scale ? option.value * option.scale : option.value);
      } else if (option.input_type == "select" && option.value && option.choices) {
        predictOptions.set(option.name, option.choices[option.value]);
      } else if (option.input_type == "text" && option.value) {
        predictOptions.set(option.name, option.value);
      } else {
        warn(`Invalid input type: ${option.input_type}`);
        message(`Invalid input type: ${option.input_type}`, { title: 'Error', type: 'error' });
      }
    });
    debug(`Initial options: ${JSON.stringify(predictOptions)}`);
    setPredictOptions(predictOptions);
  }, [model])

  useEffect(() => {
    if (!predictStatus) {
      return
    }

    if (predictStatus.stage == "finish") {
      if (predictStatus.fail) {
        error("predict fail!");
        message("解译失败！请联系技术支持人员。", {"title": "解译失败", type: "error"});
      } else {
        mapLayers.createLayer(predictStatus.params.outputPath, "vector");
      }
      setPredicting(false);
    } else {
      setPredicting(true);
    }

    if (predictStatus.stage == "loading-model") {
        setPredictMsg("加载模型");
    } else if (predictStatus.stage == "predicting") {
        setPredictMsg( "解译");
    } else if (predictStatus.stage == "postprocessing") {
        setPredictMsg("后处理");
    } else {
        setPredictMsg("后处理");
    }
  }, [predictStatus])

  return (
    <main className="flex min-h-screen flex-col h-full items-center justify-between bg-base-300">
      <div className='flex flex-row w-full grow bg-base-300 p-4'>

        <div className='flex flex-col gap-4'>
          <button
            className='btn w-32'
            disabled={predicting}
            onClick={navigatorHome}>

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
                  <input type="checkbox" onClick={onBatchModeChanged} className="toggle toggle-primary" disabled />
                </label>
              </div>
              {/* 输入 */}
              {
                model && Array.from(Array(model.input_files).keys()).map((val) => (
                  <div className='form-control' key={val}>
                    <label className='label cursor-pointer'>
                      <span className="label-text w-24">输入 {model.input_files > 1 ? val + 1 : ''}</span>
                      <div className='join'>
                        <input type="text"
                          value={inputFile[val]}
                          className="input join-item"
                          readOnly />
                        <button
                          onClick={() => { openInputFile(val) }}
                          disabled={predicting}
                          className='btn btn-neutral join-item'>选择..</button>
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
                    <input type="text"
                      value={outputFile}
                      className="input join-item" readOnly />
                    <button
                      onClick={saveOutputFile}
                      disabled={predicting}
                      className='btn btn-neutral join-item'>选择..</button>
                  </div>

                </label>
              </div>

              {/* 选择设备 */}
              <div className="form-control">
                <label className="label cursor-pointer">
                  <span className="label-text">设备</span>
                  <select
                    defaultValue="0"
                    onChange={(e) => { setDeviceId(parseInt(e.target.value)) }}
                    disabled={predicting}
                    className="select select-bordered w-full max-w-xs">
                    <option value="0">CPU</option>
                    {
                      gpuList.map((gpu, index) => (
                        <option key={index + 1} value={index + 1}>{gpu}</option>
                      )
                      )
                    }
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
                      <span className="label-text w-36">{option.label}</span>
                      {
                        option.input_type == "range" && option.min != null && option.max != null &&
                        <input
                          type="range"
                          disabled={predicting}
                          onChange={(e) => { setPredictOption(option, option.name, e.target.value) }}
                          min={option.min} max={option.max}
                          defaultValue={option.value ? option.value : option.min}
                          className={`range ${option.style}`} />
                      }
                      {
                        option.input_type == "select" &&
                        <select
                          onChange={(e) => { setPredictOption(option, option.name, e.target.value) }}
                          disabled={predicting}
                          defaultValue={option.value ? option.value : 0}
                          className={`select select-bordered w-full max-w-xs ${option.style}`}>
                          {
                            option.choices && option.choices.map((choice, index) => (
                              <option key={index} value={index}>{choice}</option>
                            )
                            )
                          }
                        </select>
                      }
                      {
                        option.input_type == "text" &&
                        <input
                          type="text"
                          disabled={predicting}
                          onChange={(e) => { setPredictOption(option, option.name, e.target.value) }}
                          placeholder=""
                          defaultValue={option.value ? option.value : undefined}
                          className="input w-full max-w-xs" />
                      }
                    </label>
                  </div>
                ))
              }

              <div className="form-control">
                <label className="label cursor-pointer">
                  <span className="label-text w-24">其它参数</span>
                  <input type="text"
                    disabled={predicting}
                    placeholder="-v --remove_small 100"
                    className="input w-full max-w-xs" />
                </label>
              </div>
            </div>

          </div>

          <div className='flex justify-end pr-6'>
            <button
              onClick={createPredictTask}
              disabled={predicting && inputFile.length == 0 || outputFile == ''}
              className='btn btn-primary w-32'>启动</button>
          </div>
        </div>

        <MapWithNoSSR/>
      </div>
      {
        predicting &&
        <div className='flex w-full'>
        <footer className="footer footer-center p-4 bg-base-300 text-base-content">
          <div className='w-full'>
            <div className='flex flex-row w-full justify-start'>

              <div className='w-32'>
                <p className="">{predictMsg}</p>
              </div>
              <div className='w-full'>
                <progress className="progress progress-primary" value={predictStatus ? predictStatus.progress * 100 : 0} max="100"></progress>
              </div>
            </div>

          </div>
        </footer>
        </div>
      }
    </main>
  )
}
