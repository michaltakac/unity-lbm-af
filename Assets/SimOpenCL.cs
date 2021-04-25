using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;

public class SimOpenCL : MonoBehaviour
{
	#if (UNITY_IOS || UNITY_TVOS || UNITY_WEBGL) && !UNITY_EDITOR
	[DllImport ("__Internal")]
	#else
	[DllImport ("lbmaf")]
	#endif
	private static extern void SetTimeFromUnity(float t);

    #if (UNITY_IOS || UNITY_TVOS || UNITY_WEBGL) && !UNITY_EDITOR
	[DllImport ("__Internal")]
	#else
	[DllImport ("lbmaf")]
	#endif
    private static extern void SetTextureFromUnity(IntPtr texture, int w, int h);

	#if (UNITY_IOS || UNITY_TVOS || UNITY_WEBGL) && !UNITY_EDITOR
	[DllImport ("__Internal")]
	#else
	[DllImport("lbmaf")]
	#endif
	private static extern IntPtr GetRenderEventFunc();

	private Texture2D tex;
    private Material material;
	public UInt32 tex_uint;

	IEnumerator Start()
	{
		tex = new Texture2D(256, 256, TextureFormat.ARGB32, false);
		tex.filterMode = FilterMode.Point;
		tex.Apply();

		GetComponent<Renderer>().material.mainTexture = tex;

        Debug.Log("Simulation started!");
		Debug.Log("tex ptr: " + tex.GetNativeTexturePtr());

        SetTextureFromUnity(tex.GetNativeTexturePtr(), tex.width, tex.height);

		yield return StartCoroutine("CallPluginAtEndOfFrames");
	}

	private IEnumerator CallPluginAtEndOfFrames()
	{
		while (true) {
			// Wait until all frame rendering is done
			yield return new WaitForEndOfFrame();

			// Set time for the plugin
			SetTimeFromUnity (Time.timeSinceLevelLoad);

			// Issue a plugin event with arbitrary integer identifier.
			// The plugin can distinguish between different
			// things it needs to do based on this ID.
			// For our simple plugin, it does not matter which ID we pass here.
			GL.IssuePluginEvent(GetRenderEventFunc(), 1);
		}
	}
	
	void Update() {
		if (Input.GetKeyDown("w"))
		{
			SendMessage("Test");
			Debug.Log(tex.GetPixel(1,1));
		}
	}
}
